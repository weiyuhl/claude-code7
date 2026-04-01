import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../services/services.dart';
import '../repositories/repositories.dart';
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
    _initSession();
  }

  void _initSession() {
    final repo = ref.read(sessionRepositoryProvider);
    final success = repo.createSession(
      provider: state.currentProvider,
      model: state.currentModel.isEmpty ? 'auto' : state.currentModel,
    );
    if (!success) {
      throw Exception('Failed to create session');
    }
  }

  void sendMessage(String text) {
    if (text.isEmpty || state.isStreaming) return;

    final apiKey = state.apiKeys[state.currentProvider] ?? '';
    if (apiKey.isEmpty) {
      throw Exception('Please configure API Key in settings');
    }

    final repo = ref.read(sessionRepositoryProvider);
    repo.setProvider(state.currentProvider, apiKey);

    state = state.copyWith(
      messages: [
        ...state.messages,
        {'role': 'user', 'content': text},
        {'role': 'assistant', 'content': '', 'thinking': '', 'tool_use': null},
      ],
      isStreaming: true,
    );

    try {
      String fullContent = '';
      String fullThinking = '';

      repo.streamMessage(text, (chunk) {
        final type = chunk['type'];
        final content = chunk['content'];

        if (type == 'content') {
          fullContent += content;
        } else if (type == 'thinking') {
          fullThinking += content;
        }

        final updatedMessages = List<Map<String, dynamic>>.from(state.messages);
        updatedMessages[updatedMessages.length - 1] = {
          'role': 'assistant',
          'content': fullContent,
          'thinking': fullThinking,
          'tool_use': type == 'tool_use' ? content : null,
        };

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
    final success = repo.createSession(
      provider: state.currentProvider,
      model: state.currentModel.isEmpty ? 'auto' : state.currentModel,
    );
    if (success) {
      final apiKey = state.apiKeys[state.currentProvider] ?? '';
      if (apiKey.isNotEmpty) {
        repo.setProvider(state.currentProvider, apiKey);
        _refreshInfo();
      }
    }
  }

  void _refreshInfo() {
    final repo = ref.read(sessionRepositoryProvider);
    final apiKey = state.apiKeys[state.currentProvider] ?? '';
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
      print('Failed to fetch models: $e');
    }

    try {
      final balanceInfo = repo.getBalance();
      state = state.copyWith(balance: balanceInfo);
    } catch (e) {
      print('Failed to fetch balance: $e');
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
