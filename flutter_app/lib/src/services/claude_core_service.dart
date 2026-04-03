import 'dart:ffi';
import 'dart:io';
import 'dart:convert';
import 'package:flutter/foundation.dart';
import 'package:ffi/ffi.dart';

final calloc = _Calloc();

class _Calloc implements Allocator {
  @override
  Pointer<T> allocate<T extends NativeType>(int byteCount, {int? alignment}) {
    return malloc.allocate<T>(byteCount, alignment: alignment);
  }

  @override
  void free(Pointer pointer) {
    malloc.free(pointer);
  }
}

class ClaudeCore {
  late DynamicLibrary _lib;

  late int Function(Pointer<Utf8> dbPath) _initDatabase;
  late Pointer<Void> Function(Pointer<Utf8> configJson) _createSession;
  late Pointer<Utf8> Function(Pointer<Void> session, Pointer<Utf8> content)
      _sendMessage;
  late void Function(Pointer<Void> session) _destroySession;
  late Pointer<Utf8> Function(Pointer<Void> session) _getMessages;
  late Pointer<Utf8> Function(Pointer<Void> session) _listModels;
  late Pointer<Utf8> Function(Pointer<Void> session) _getBalance;
  late void Function(Pointer<Utf8> s) _freeString;
  late bool Function(
    Pointer<Void> session,
    Pointer<Utf8> providerName,
    Pointer<Utf8> apiKey,
  ) _setProvider;

  late int Function(
    Pointer<Void> session,
    Pointer<Utf8> content,
    Pointer<NativeFunction<Void Function(Pointer<Utf8>, Pointer<Void>)>>
        callback,
    Pointer<Void> userData,
  ) _streamMessage;

  late int Function(Pointer<Utf8> provider, Pointer<Utf8> apiKey) _setApiKey;
  late Pointer<Utf8> Function(Pointer<Utf8> provider) _getApiKey;
  late int Function(
    Pointer<Void> session,
    Pointer<Utf8> summary,
    Pointer<Utf8> boundaryMsgId,
  ) _compactSession;
  late Pointer<Utf8> Function(Pointer<Void> session) _getConversationHistory;

  NativeCallable<Void Function(Pointer<Utf8>, Pointer<Void>)>?
      _streamCallbackRef;

  ClaudeCore() {
    if (Platform.isAndroid) {
      _lib = DynamicLibrary.open('libclaude_core.so');
    } else if (Platform.isWindows) {
      _lib = DynamicLibrary.open('claude_core.dll');
    } else if (Platform.isLinux) {
      _lib = DynamicLibrary.open('libclaude_core.so');
    } else if (Platform.isMacOS) {
      _lib = DynamicLibrary.open('libclaude_core.dylib');
    } else if (Platform.isIOS) {
      _lib = DynamicLibrary.process();
    } else {
      throw UnsupportedError(
        'Unsupported platform: ${Platform.operatingSystem}',
      );
    }

    _initDatabase = _lib
        .lookup<NativeFunction<Int32 Function(Pointer<Utf8>)>>('init_database')
        .asFunction();

    _createSession = _lib
        .lookup<NativeFunction<Pointer<Void> Function(Pointer<Utf8>)>>(
          'create_session',
        )
        .asFunction();

    _sendMessage = _lib
        .lookup<
            NativeFunction<
                Pointer<Utf8> Function(Pointer<Void>, Pointer<Utf8>)>>(
          'send_message',
        )
        .asFunction();

    _destroySession = _lib
        .lookup<NativeFunction<Void Function(Pointer<Void>)>>('destroy_session')
        .asFunction();

    _getMessages = _lib
        .lookup<NativeFunction<Pointer<Utf8> Function(Pointer<Void>)>>(
          'get_messages',
        )
        .asFunction();

    _listModels = _lib
        .lookup<NativeFunction<Pointer<Utf8> Function(Pointer<Void>)>>(
          'list_models',
        )
        .asFunction();

    _getBalance = _lib
        .lookup<NativeFunction<Pointer<Utf8> Function(Pointer<Void>)>>(
          'get_balance',
        )
        .asFunction();

    _freeString = _lib
        .lookup<NativeFunction<Void Function(Pointer<Utf8>)>>('free_string')
        .asFunction();

    _setProvider = _lib
        .lookup<
            NativeFunction<
                Bool Function(Pointer<Void>, Pointer<Utf8>, Pointer<Utf8>)>>(
          'set_provider',
        )
        .asFunction();

    _streamMessage = _lib
        .lookup<
            NativeFunction<
                Int32 Function(
                  Pointer<Void>,
                  Pointer<Utf8>,
                  Pointer<
                      NativeFunction<
                          Void Function(Pointer<Utf8>, Pointer<Void>)>>,
                  Pointer<Void>,
                )>>('stream_message')
        .asFunction();

    _setApiKey = _lib
        .lookup<NativeFunction<Int32 Function(Pointer<Utf8>, Pointer<Utf8>)>>(
          'set_api_key',
        )
        .asFunction();

    _getApiKey = _lib
        .lookup<NativeFunction<Pointer<Utf8> Function(Pointer<Utf8>)>>(
          'get_api_key',
        )
        .asFunction();

    _compactSession = _lib
        .lookup<
            NativeFunction<
                Int32 Function(Pointer<Void>, Pointer<Utf8>, Pointer<Utf8>)>>(
          'compact_session',
        )
        .asFunction();

    _getConversationHistory = _lib
        .lookup<NativeFunction<Pointer<Utf8> Function(Pointer<Void>)>>(
          'get_conversation_history',
        )
        .asFunction();
  }

  Pointer<Void> createSession(Map<String, dynamic> config) {
    final configJson = jsonEncode(config);
    debugPrint('🔵 [ClaudeCore.createSession] configJson: $configJson');
    final configPtr = configJson.toNativeUtf8();
    final result = _createSession(configPtr);
    debugPrint('🔵 [ClaudeCore.createSession] result: $result');
    calloc.free(configPtr);
    return result;
  }

  String sendMessage(Pointer<Void> session, String content) {
    final contentPtr = content.toNativeUtf8();
    try {
      final resultPtr = _sendMessage(session, contentPtr);
      try {
        return resultPtr.toDartString();
      } finally {
        _freeString(resultPtr);
      }
    } finally {
      calloc.free(contentPtr);
    }
  }

  void streamMessage(
    Pointer<Void> session,
    String content,
    void Function(Map<String, dynamic>) onChunk,
  ) {
    debugPrint(
        '🔵 [ClaudeCoreService.streamMessage] 开始 - content: $content');

    final contentPtr = content.toNativeUtf8();
    final nativeCallable =
        NativeCallable<Void Function(Pointer<Utf8>, Pointer<Void>)>.listener((
      Pointer<Utf8> chunkPtr,
      Pointer<Void> userData,
    ) {
      final chunkStr = chunkPtr.toDartString();
      debugPrint(
          '🔵 [ClaudeCoreService.streamMessage] 收到 chunk: $chunkStr');
      // Free the leaked CString from Rust
      _freeString(chunkPtr);
      try {
        final chunk = jsonDecode(chunkStr) as Map<String, dynamic>;
        onChunk(chunk);
      } catch (e) {
        onChunk({"type": "content", "content": chunkStr});
      }
    });

    _streamCallbackRef = nativeCallable;

    try {
      debugPrint('🔵 [ClaudeCoreService.streamMessage] 调用 _streamMessage');
      _streamMessage(
        session,
        contentPtr,
        nativeCallable.nativeFunction,
        nullptr,
      );
      debugPrint(
          '🔵 [ClaudeCoreService.streamMessage] _streamMessage 调用完成');
    } finally {
      calloc.free(contentPtr);
    }
  }

  void destroySession(Pointer<Void> session) {
    _streamCallbackRef?.close();
    _streamCallbackRef = null;
    _destroySession(session);
  }

  String getMessages(Pointer<Void> session) {
    final resultPtr = _getMessages(session);
    try {
      return resultPtr.toDartString();
    } finally {
      _freeString(resultPtr);
    }
  }

  List<dynamic> listModels(Pointer<Void> session) {
    if (session == nullptr) {
      throw Exception('listModels: null session pointer');
    }
    final resultPtr = _listModels(session);
    if (resultPtr == nullptr) {
      throw Exception('listModels: null pointer returned from Rust');
    }
    try {
      final jsonStr = resultPtr.toDartString();
      final decoded = jsonDecode(jsonStr);
      if (decoded is Map && decoded.containsKey('error')) {
        throw Exception('listModels error: ${decoded['error']}');
      }
      return decoded is List ? decoded : [];
    } finally {
      _freeString(resultPtr);
    }
  }

  Map<String, dynamic> getBalance(Pointer<Void> session) {
    if (session == nullptr) {
      throw Exception('getBalance: null session pointer');
    }
    final resultPtr = _getBalance(session);
    if (resultPtr == nullptr) {
      throw Exception('getBalance: null pointer returned from Rust');
    }
    try {
      final jsonStr = resultPtr.toDartString();
      final decoded = jsonDecode(jsonStr);
      if (decoded is Map && decoded.containsKey('error')) {
        throw Exception('getBalance error: ${decoded['error']}');
      }
      return decoded is Map ? Map<String, dynamic>.from(decoded) : {};
    } finally {
      _freeString(resultPtr);
    }
  }

  int initDatabase(String dbPath) {
    final dbPathPtr = dbPath.toNativeUtf8();
    try {
      return _initDatabase(dbPathPtr);
    } finally {
      calloc.free(dbPathPtr);
    }
  }

  bool setProvider(Pointer<Void> session, String providerName, String apiKey) {
    final providerNamePtr = providerName.toNativeUtf8();
    final apiKeyPtr = apiKey.toNativeUtf8();
    try {
      return _setProvider(session, providerNamePtr, apiKeyPtr);
    } finally {
      calloc.free(providerNamePtr);
      calloc.free(apiKeyPtr);
    }
  }

  int setApiKey(String provider, String apiKey) {
    final providerPtr = provider.toNativeUtf8();
    final apiKeyPtr = apiKey.toNativeUtf8();
    try {
      return _setApiKey(providerPtr, apiKeyPtr);
    } finally {
      calloc.free(providerPtr);
      calloc.free(apiKeyPtr);
    }
  }

  String? getApiKey(String provider) {
    final providerPtr = provider.toNativeUtf8();
    try {
      final resultPtr = _getApiKey(providerPtr);
      try {
        final key = resultPtr.toDartString();
        return key.isEmpty ? null : key;
      } finally {
        _freeString(resultPtr);
      }
    } finally {
      calloc.free(providerPtr);
    }
  }

  int compactSession(
    Pointer<Void> session,
    String summary,
    String boundaryMsgId,
  ) {
    final summaryPtr = summary.toNativeUtf8();
    final boundaryMsgIdPtr = boundaryMsgId.toNativeUtf8();
    try {
      return _compactSession(session, summaryPtr, boundaryMsgIdPtr);
    } finally {
      calloc.free(summaryPtr);
      calloc.free(boundaryMsgIdPtr);
    }
  }

  String getConversationHistory(Pointer<Void> session) {
    final resultPtr = _getConversationHistory(session);
    try {
      return resultPtr.toDartString();
    } finally {
      _freeString(resultPtr);
    }
  }
}
