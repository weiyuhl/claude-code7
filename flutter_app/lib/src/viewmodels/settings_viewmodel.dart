import 'dart:ffi';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../services/services.dart';
import 'providers.dart';

class SettingsState {
  final String selectedProvider;
  final String selectedModel;
  final String apiKey;
  final List<String> models;
  final Map<String, dynamic> balance;
  final bool isLoadingModels;
  final bool isLoadingBalance;

  const SettingsState({
    this.selectedProvider = 'openrouter',
    this.selectedModel = '',
    this.apiKey = '',
    this.models = const [],
    this.balance = const {},
    this.isLoadingModels = false,
    this.isLoadingBalance = false,
  });

  SettingsState copyWith({
    String? selectedProvider,
    String? selectedModel,
    String? apiKey,
    List<String>? models,
    Map<String, dynamic>? balance,
    bool? isLoadingModels,
    bool? isLoadingBalance,
  }) {
    return SettingsState(
      selectedProvider: selectedProvider ?? this.selectedProvider,
      selectedModel: selectedModel ?? this.selectedModel,
      apiKey: apiKey ?? this.apiKey,
      models: models ?? this.models,
      balance: balance ?? this.balance,
      isLoadingModels: isLoadingModels ?? this.isLoadingModels,
      isLoadingBalance: isLoadingBalance ?? this.isLoadingBalance,
    );
  }
}

class SettingsNotifier extends StateNotifier<SettingsState> {
  final Ref ref;
  Pointer<Void>? _tempSession;

  SettingsNotifier(this.ref) : super(const SettingsState()) {
    _createTempSession();
  }

  void _createTempSession() {
    final claudeCore = ref.read(claudeCoreProvider);
    _destroyTempSession();

    final config = {
      'provider': state.selectedProvider,
      'model': 'auto',
      'max_tokens': 4096,
    };

    _tempSession = claudeCore.createSession(config);
  }

  void _destroyTempSession() {
    if (_tempSession != null && _tempSession != nullptr) {
      final claudeCore = ref.read(claudeCoreProvider);
      claudeCore.destroySession(_tempSession!);
      _tempSession = null;
    }
  }

  void updateProvider(String provider, String apiKey) {
    _destroyTempSession();
    state = state.copyWith(
      selectedProvider: provider,
      apiKey: apiKey,
      selectedModel: '',
      models: [],
    );
    _createTempSession();
  }

  void updateApiKey(String key) {
    state = state.copyWith(apiKey: key);
  }

  Future<void> fetchModels() async {
    if (state.apiKey.isEmpty) {
      throw Exception('Please enter API Key');
    }

    state = state.copyWith(isLoadingModels: true);

    try {
      final claudeCore = ref.read(claudeCoreProvider);
      if (_tempSession == null || _tempSession == nullptr) {
        throw Exception('No active session');
      }

      final success = claudeCore.setProvider(
        _tempSession!,
        state.selectedProvider,
        state.apiKey,
      );
      if (!success) {
        throw Exception('Failed to set provider');
      }

      final modelsList = claudeCore.listModels(_tempSession!);
      final modelIds = modelsList.map((m) => m['id'].toString()).toList();

      state = state.copyWith(
        models: modelIds,
        selectedModel: modelIds.isNotEmpty ? modelIds.first : '',
      );
    } finally {
      state = state.copyWith(isLoadingModels: false);
    }
  }

  Future<void> fetchBalance() async {
    if (state.apiKey.isEmpty) {
      throw Exception('Please enter API Key');
    }

    state = state.copyWith(isLoadingBalance: true);

    try {
      final claudeCore = ref.read(claudeCoreProvider);
      if (_tempSession == null || _tempSession == nullptr) {
        throw Exception('No active session');
      }

      claudeCore.setProvider(
        _tempSession!,
        state.selectedProvider,
        state.apiKey,
      );

      final balanceInfo = claudeCore.getBalance(_tempSession!);
      state = state.copyWith(balance: balanceInfo);
    } finally {
      state = state.copyWith(isLoadingBalance: false);
    }
  }

  void selectModel(String model) {
    state = state.copyWith(selectedModel: model);
  }

  void dispose() {
    _destroyTempSession();
  }
}

final settingsNotifierProvider =
    StateNotifierProvider<SettingsNotifier, SettingsState>((ref) {
      return SettingsNotifier(ref);
    });
