import 'dart:ffi';
import '../services/services.dart';

class SessionRepository {
  final ClaudeCore _claudeCore;
  Pointer<Void>? _session;

  SessionRepository(this._claudeCore);

  Pointer<Void>? get session => _session;

  bool createSession({
    required String provider,
    required String model,
    int maxTokens = 4096,
    String? apiKey,
  }) {
    destroySession();

    final config = {
      'provider': provider,
      'model': model,
      'max_tokens': maxTokens,
    };

    _session = _claudeCore.createSession(config);
    if (_session == null || _session == nullptr) return false;

    // If API key is provided, set the provider immediately
    if (apiKey != null && apiKey.isNotEmpty) {
      _claudeCore.setProvider(_session!, provider, apiKey);
    }

    return true;
  }

  void destroySession() {
    if (_session != null && _session != nullptr) {
      _claudeCore.destroySession(_session!);
      _session = null;
    }
  }

  bool setProvider(String providerName, String apiKey) {
    if (_session == null || _session == nullptr) return false;
    return _claudeCore.setProvider(_session!, providerName, apiKey);
  }

  List<dynamic> listModels() {
    if (_session == null || _session == nullptr) {
      throw Exception('No active session');
    }
    return _claudeCore.listModels(_session!);
  }

  Map<String, dynamic> getBalance() {
    if (_session == null || _session == nullptr) {
      throw Exception('No active session');
    }
    return _claudeCore.getBalance(_session!);
  }

  void streamMessage(
    String content,
    void Function(Map<String, dynamic>) onChunk,
  ) {
    if (_session == null || _session == nullptr) {
      throw Exception('No active session');
    }
    _claudeCore.streamMessage(_session!, content, onChunk);
  }

  String getMessages() {
    if (_session == null || _session == nullptr) {
      throw Exception('No active session');
    }
    return _claudeCore.getMessages(_session!);
  }

  String getConversationHistory() {
    if (_session == null || _session == nullptr) {
      throw Exception('No active session');
    }
    return _claudeCore.getConversationHistory(_session!);
  }

  int compactSession(String summary, String boundaryMsgId) {
    if (_session == null || _session == nullptr) {
      throw Exception('No active session');
    }
    return _claudeCore.compactSession(_session!, summary, boundaryMsgId);
  }

  int setApiKey(String provider, String apiKey) {
    return _claudeCore.setApiKey(provider, apiKey);
  }

  String? getApiKey(String provider) {
    return _claudeCore.getApiKey(provider);
  }

  void dispose() {
    destroySession();
  }
}
