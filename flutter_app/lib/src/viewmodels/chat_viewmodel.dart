import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'providers.dart';

class ChatState {
  final List<Map<String, dynamic>> messages;
  final bool isStreaming;
  final String currentProvider;
  final String currentModel;
  final Map<String, String> apiKeys;
  final Map<String, List<String>> models;
  final Map<String, dynamic> balance;

  const ChatState({
    this.messages = const [],
    this.isStreaming = false,
    this.currentProvider = 'openrouter',
    this.currentModel = '',
    this.apiKeys = const {'openrouter': '', 'deepseek': '', 'siliconflow': ''},
    this.models = const {'openrouter': [], 'deepseek': [], 'siliconflow': []},
    this.balance = const {
      'is_available': false,
      'total_balance': '0.00',
      'currency': 'USD',
    },
  });

  ChatState copyWith({
    List<Map<String, dynamic>>? messages,
    bool? isStreaming,
    String? currentProvider,
    String? currentModel,
    Map<String, String>? apiKeys,
    Map<String, List<String>>? models,
    Map<String, dynamic>? balance,
  }) {
    return ChatState(
      messages: messages ?? this.messages,
      isStreaming: isStreaming ?? this.isStreaming,
      currentProvider: currentProvider ?? this.currentProvider,
      currentModel: currentModel ?? this.currentModel,
      apiKeys: apiKeys ?? this.apiKeys,
      models: models ?? this.models,
      balance: balance ?? this.balance,
    );
  }
}

class ChatNotifier extends StateNotifier<ChatState> {
  final Ref ref;

  ChatNotifier(this.ref) : super(const ChatState()) {
    _loadApiKeyAndInit();
  }

  Future<void> _loadApiKeyAndInit() async {
    await loadApiKeyFromStorage();
    _initSession();
  }

  void _initSession() {
    final repo = ref.read(sessionRepositoryProvider);
    final apiKey = repo.getApiKey(state.currentProvider) ?? '';
    final success = repo.createSession(
      provider: state.currentProvider,
      model: state.currentModel.isEmpty ? 'auto' : state.currentModel,
      apiKey: apiKey,
    );
    if (!success) {
      throw Exception('Failed to create session');
    }
  }

  void sendMessage(String text) {
    if (text.isEmpty || state.isStreaming) return;

    final repo = ref.read(sessionRepositoryProvider);
    // Always read API key from DB, not just in-memory state
    final apiKey =
        repo.getApiKey(state.currentProvider) ??
        state.apiKeys[state.currentProvider] ??
        '';
    if (apiKey.isEmpty) {
      throw Exception('Please configure API Key in settings');
    }

    repo.setProvider(state.currentProvider, apiKey);

    // Pre-add user and empty assistant messages
    state = state.copyWith(
      messages: [
        ...state.messages,
        {'role': 'user', 'content': text},
        {'role': 'assistant', 'content': '', 'thinking': '', 'tool_use': null},
      ],
      isStreaming: true,
    );

    // Use local accumulators to avoid stale closure — the callback captures
    // these mutable variables, not the immutable `state`.
    String fullContent = '';
    String fullThinking = '';
    dynamic lastToolUse;

    try {
      repo.streamMessage(text, (chunk) {
        final type = chunk['type'];
        final content = chunk['content'];

        if (type == 'content') {
          fullContent += content;
        } else if (type == 'thinking') {
          fullThinking += content;
        } else if (type == 'tool_use') {
          lastToolUse = content;
        } else if (type == 'error') {
          // Handle error events from Rust SSE stream
          state = state.copyWith(isStreaming: false);
          throw Exception('Stream error: $content');
        }

        // Build a fresh list each time — avoids stale closure on state.messages
        final updatedMessages = List<Map<String, dynamic>>.from(state.messages);
        if (updatedMessages.isNotEmpty) {
          updatedMessages[updatedMessages.length - 1] = {
            'role': 'assistant',
            'content': fullContent,
            'thinking': fullThinking,
            'tool_use': lastToolUse,
          };
        }

        state = state.copyWith(messages: updatedMessages);
      });

      state = state.copyWith(isStreaming: false);
    } catch (e) {
      state = state.copyWith(isStreaming: false);
      rethrow;
    }
  }

  void updateProvider(String provider) {
    state = state.copyWith(currentProvider: provider, currentModel: '');
    _recreateSession();
  }

  void updateModel(String model) {
    state = state.copyWith(currentModel: model);
    _recreateSession();
  }

  void updateApiKey(String provider, String key) {
    final newKeys = Map<String, String>.from(state.apiKeys);
    newKeys[provider] = key;
    state = state.copyWith(apiKeys: newKeys);
  }

  void _recreateSession() {
    final repo = ref.read(sessionRepositoryProvider);
    // Read API key from DB, not in-memory state
    final apiKey =
        repo.getApiKey(state.currentProvider) ??
        state.apiKeys[state.currentProvider] ??
        '';
    final success = repo.createSession(
      provider: state.currentProvider,
      model: state.currentModel.isEmpty ? 'auto' : state.currentModel,
      apiKey: apiKey,
    );
    if (success && apiKey.isNotEmpty) {
      _refreshInfo();
    }
  }

  void _refreshInfo() {
    final repo = ref.read(sessionRepositoryProvider);
    // Read API key from DB, not in-memory state
    final apiKey =
        repo.getApiKey(state.currentProvider) ??
        state.apiKeys[state.currentProvider] ??
        '';
    if (apiKey.isEmpty) return;

    try {
      final modelsList = repo.listModels();
      if (modelsList.isNotEmpty) {
        final modelIds = modelsList.map((m) => m['id'].toString()).toList();
        final newModels = Map<String, List<String>>.from(state.models);
        newModels[state.currentProvider] = modelIds;

        String newModel = state.currentModel;
        if (newModel.isEmpty && modelIds.isNotEmpty) {
          newModel = modelIds.first;
        }

        state = state.copyWith(models: newModels, currentModel: newModel);
      }
    } catch (e) {
      debugPrint('Failed to fetch models: $e');
    }

    try {
      final balanceInfo = repo.getBalance();
      state = state.copyWith(balance: balanceInfo);
    } catch (e) {
      debugPrint('Failed to fetch balance: $e');
    }
  }

  Future<void> loadHistory() async {
    final repo = ref.read(sessionRepositoryProvider);
    try {
      final historyJson = repo.getConversationHistory();
      if (historyJson.isEmpty) return;

      final decoded = jsonDecode(historyJson) as Map<String, dynamic>;
      final rawMessages = decoded['messages'] as List?;
      if (rawMessages == null || rawMessages.isEmpty) return;

      // Normalize Rust Message struct fields to Flutter UI format
      final messageList = rawMessages.map((m) {
        final map = Map<String, dynamic>.from(m);
        // Ensure thinking field exists (Rust Message may not have it in older data)
        map['thinking'] = map['thinking'] ?? '';
        map['tool_use'] = map['tool_use'] ?? map['tool_name'];
        return map;
      }).toList();

      state = state.copyWith(messages: messageList);
    } catch (e) {
      debugPrint('Failed to load history: $e');
    }
  }

  Future<int> compactContext(String summary, String boundaryMsgId) async {
    final repo = ref.read(sessionRepositoryProvider);
    final result = repo.compactSession(summary, boundaryMsgId);
    if (result == 0) {
      await loadHistory();
    }
    return result;
  }

  Future<void> loadApiKeyFromStorage() async {
    final repo = ref.read(sessionRepositoryProvider);
    final provider = state.currentProvider;
    final key = repo.getApiKey(provider);
    if (key != null) {
      updateApiKey(provider, key);
    }
  }

  @override
  void dispose() {
    ref.read(sessionRepositoryProvider).dispose();
    super.dispose();
  }
}

final chatNotifierProvider = StateNotifierProvider<ChatNotifier, ChatState>((
  ref,
) {
  return ChatNotifier(ref);
});
