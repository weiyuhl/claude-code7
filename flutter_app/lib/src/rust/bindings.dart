import 'dart:ffi';
import 'dart:io';
import 'dart:convert';
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

  late Pointer<Void> Function(Pointer<Utf8> configJson) _createSession;
  late Pointer<Utf8> Function(Pointer<Void> session, Pointer<Utf8> content) _sendMessage;
  late void Function(Pointer<Void> session) _destroySession;
  late Pointer<Utf8> Function(Pointer<Void> session) _getMessages;
  late void Function(Pointer<Utf8> s) _freeString;
  late bool Function(Pointer<Void> session, Pointer<Utf8> providerName, Pointer<Utf8> apiKey) _setProvider;

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
      throw UnsupportedError('Unsupported platform: ${Platform.operatingSystem}');
    }

    _createSession = _lib
        .lookup<NativeFunction<Pointer<Void> Function(Pointer<Utf8>)>>('claude_create_session')
        .asFunction();

    _sendMessage = _lib
        .lookup<NativeFunction<Pointer<Utf8> Function(Pointer<Void>, Pointer<Utf8>)>>('claude_send_message')
        .asFunction();

    _destroySession = _lib
        .lookup<NativeFunction<Void Function(Pointer<Void>)>>('claude_destroy_session')
        .asFunction();

    _getMessages = _lib
        .lookup<NativeFunction<Pointer<Utf8> Function(Pointer<Void>)>>('claude_get_messages')
        .asFunction();

    _freeString = _lib
        .lookup<NativeFunction<Void Function(Pointer<Utf8>)>>('claude_free_string')
        .asFunction();

    _setProvider = _lib
        .lookup<NativeFunction<Bool Function(Pointer<Void>, Pointer<Utf8>, Pointer<Utf8>)>>('set_provider')
        .asFunction();
  }

  Pointer<Void> createSession(Map<String, dynamic> config) {
    final configJson = jsonEncode(config);
    final configPtr = configJson.toNativeUtf8();
    try {
      return _createSession(configPtr);
    } finally {
      calloc.free(configPtr);
    }
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

  void destroySession(Pointer<Void> session) {
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
}
