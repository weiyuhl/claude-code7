import 'dart:ffi';
import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path_provider/path_provider.dart';
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
  File? _logFile;

  SettingsNotifier(this.ref) : super(const SettingsState()) {
    _initLogFile();
    _loadSavedConfigAndInit();
  }

  Future<void> _initLogFile() async {
    try {
      // 使用外部存储目录，不需要 root 权限即可访问
      final directory = await getExternalStorageDirectory();
      if (directory != null) {
        _logFile = File('${directory.path}/claude_app.log');
        await _writeLog('=== 应用启动 ===\n');
        debugPrint('📁 日志文件路径：${_logFile!.path}');
      }
    } catch (e) {
      debugPrint('❌ 初始化日志文件失败：$e');
    }
  }

  Future<void> _writeLog(String message) async {
    if (_logFile != null) {
      try {
        final timestamp = DateTime.now().toString();
        await _logFile!.writeAsString('[$timestamp] $message\n', mode: FileMode.append);
      } catch (e) {
        debugPrint('❌ 写入日志失败：$e');
      }
    }
  }

  Future<void> _loadSavedConfigAndInit() async {
    final repo = ref.read(sessionRepositoryProvider);
    final savedApiKey = repo.getApiKey(state.selectedProvider);

    final log1 = '🔵 [SettingsNotifier] 加载 API Key - Provider: ${state.selectedProvider}, 结果：${savedApiKey ?? "null"}';
    debugPrint(log1);
    await _writeLog(log1);

    if (savedApiKey != null && savedApiKey.isNotEmpty) {
      state = state.copyWith(apiKey: savedApiKey);
      final log2 = '🔵 [SettingsNotifier] 已更新 state.apiKey';
      debugPrint(log2);
      await _writeLog(log2);
    }

    _createTempSession();
    
    final log3 = '🔵 [SettingsNotifier] 创建会话后 - _tempSession: ${_tempSession != null}, apiKey: ${state.apiKey.isEmpty ? "空" : "有值"}';
    debugPrint(log3);
    await _writeLog(log3);
    
    // 如果有 API Key，设置到 session 中
    if (state.apiKey.isNotEmpty && _tempSession != null && _tempSession != nullptr) {
      final claudeCore = ref.read(claudeCoreProvider);
      final result = claudeCore.setProvider(_tempSession!, state.selectedProvider, state.apiKey);
      final log4 = '🔵 [SettingsNotifier] 设置 Provider 结果：$result';
      debugPrint(log4);
      await _writeLog(log4);
    }
  }

  void _createTempSession() {
    final log1 = '🔵 [SettingsNotifier] _createTempSession 开始 - apiKey: ${state.apiKey.isEmpty ? "空" : "有值"}';
    debugPrint(log1);
    _writeLog(log1);
    
    final claudeCore = ref.read(claudeCoreProvider);
    _destroyTempSession();

    final config = {
      'provider': state.selectedProvider,
      'model': 'auto',
      'max_tokens': 4096,
    };

    debugPrint('🔵 [SettingsNotifier] 创建会话，config: $config');
    _writeLog('🔵 [SettingsNotifier] 创建会话，config: $config');

    _tempSession = claudeCore.createSession(config);
    final log2 = '🔵 [SettingsNotifier] 创建会话结果：${_tempSession != null}';
    debugPrint(log2);
    _writeLog(log2);
    
    // 设置 provider 和 API Key
    if (_tempSession != null && _tempSession != nullptr && state.apiKey.isNotEmpty) {
      final result = claudeCore.setProvider(_tempSession!, state.selectedProvider, state.apiKey);
      final log3 = '🔵 [SettingsNotifier] 设置 Provider 结果：$result';
      debugPrint(log3);
      _writeLog(log3);
    } else {
      final log4 = '🔵 [SettingsNotifier] 跳过设置 Provider - session: ${_tempSession != null}, apiKey 有值：${state.apiKey.isNotEmpty}';
      debugPrint(log4);
      _writeLog(log4);
    }
  }

  void _destroyTempSession() {
    if (_tempSession != null && _tempSession != nullptr) {
      final claudeCore = ref.read(claudeCoreProvider);
      claudeCore.destroySession(_tempSession!);
      _tempSession = null;
    }
  }

  Future<void> updateProvider(String provider, String apiKey) async {
    _destroyTempSession();
    final repo = ref.read(sessionRepositoryProvider);
    final savedKeyForNewProvider = repo.getApiKey(provider);
    final keyToUse =
        (savedKeyForNewProvider != null && savedKeyForNewProvider.isNotEmpty)
        ? savedKeyForNewProvider
        : apiKey;
    state = state.copyWith(
      selectedProvider: provider,
      apiKey: keyToUse,
      selectedModel: '',
      models: [],
    );
    if (keyToUse.isNotEmpty) {
      repo.setApiKey(provider, keyToUse);
    }
    _createTempSession();
  }

  void updateApiKey(String key) {
    state = state.copyWith(apiKey: key);
    final repo = ref.read(sessionRepositoryProvider);
    repo.setApiKey(state.selectedProvider, key);
  }

  Future<void> fetchModels() async {
    final log1 = '🔵 [fetchModels] 开始 - apiKey: ${state.apiKey.isEmpty ? "空" : "有值"}, _tempSession: ${_tempSession != null}';
    debugPrint(log1);
    await _writeLog(log1);
    
    if (state.apiKey.isEmpty) {
      final log2 = '🔴 [fetchModels] API Key 为空，抛出异常';
      debugPrint(log2);
      await _writeLog(log2);
      throw Exception('Please enter API Key');
    }

    state = state.copyWith(isLoadingModels: true);

    try {
      final claudeCore = ref.read(claudeCoreProvider);
      if (_tempSession == null || _tempSession == nullptr) {
        final log3 = '🔵 [fetchModels] 会话为空，创建新会话';
        debugPrint(log3);
        await _writeLog(log3);
        _createTempSession();
        final log3b = '🔵 [fetchModels] _createTempSession 返回后 - _tempSession: ${_tempSession != null}, == nullptr: ${_tempSession == nullptr}';
        debugPrint(log3b);
        await _writeLog(log3b);
      }
      final log4a = '🔵 [fetchModels] 第二次检查前 - _tempSession: ${_tempSession != null}, == nullptr: ${_tempSession == nullptr}';
      debugPrint(log4a);
      await _writeLog(log4a);
      if (_tempSession == null || _tempSession == nullptr) {
        final log4 = '🔴 [fetchModels] 会话仍然为空，抛出异常 - _tempSession: $_tempSession';
        debugPrint(log4);
        await _writeLog(log4);
        throw Exception('No active session');
      }

      final log5 = '🔵 [fetchModels] 设置 Provider: ${state.selectedProvider}';
      debugPrint(log5);
      await _writeLog(log5);
      final success = claudeCore.setProvider(
        _tempSession!,
        state.selectedProvider,
        state.apiKey,
      );
      if (!success) {
        final log6 = '🔴 [fetchModels] 设置 Provider 失败';
        debugPrint(log6);
        await _writeLog(log6);
        throw Exception('Failed to set provider');
      }

      final log7 = '🔵 [fetchModels] 获取模型列表';
      debugPrint(log7);
      await _writeLog(log7);
      final modelsList = claudeCore.listModels(_tempSession!);
      final modelIds = modelsList.map((m) => m['id'].toString()).toList();
      final log8 = '🟢 [fetchModels] 获取到 ${modelIds.length} 个模型';
      debugPrint(log8);
      await _writeLog(log8);

      state = state.copyWith(
        models: modelIds,
        selectedModel: modelIds.isNotEmpty ? modelIds.first : '',
      );
    } catch (e) {
      final log9 = '🔴 [fetchModels] 异常：$e';
      debugPrint(log9);
      await _writeLog(log9);
      state = state.copyWith(models: [], selectedModel: '');
      rethrow;
    } finally {
      state = state.copyWith(isLoadingModels: false);
    }
  }

  Future<void> fetchBalance() async {
    final log1 = '🔵 [fetchBalance] 开始 - apiKey: ${state.apiKey.isEmpty ? "空" : "有值"}, _tempSession: ${_tempSession != null}';
    debugPrint(log1);
    await _writeLog(log1);
    
    if (state.apiKey.isEmpty) {
      final log2 = '🔴 [fetchBalance] API Key 为空，抛出异常';
      debugPrint(log2);
      await _writeLog(log2);
      throw Exception('Please enter API Key');
    }

    state = state.copyWith(isLoadingBalance: true);

    try {
      final claudeCore = ref.read(claudeCoreProvider);
      if (_tempSession == null || _tempSession == nullptr) {
        final log3 = '🔵 [fetchBalance] 会话为空，创建新会话';
        debugPrint(log3);
        await _writeLog(log3);
        _createTempSession();
      }
      if (_tempSession == null || _tempSession == nullptr) {
        final log4 = '🔴 [fetchBalance] 会话仍然为空，抛出异常';
        debugPrint(log4);
        await _writeLog(log4);
        throw Exception('No active session');
      }

      final log5 = '🔵 [fetchBalance] 设置 Provider: ${state.selectedProvider}';
      debugPrint(log5);
      await _writeLog(log5);
      claudeCore.setProvider(
        _tempSession!,
        state.selectedProvider,
        state.apiKey,
      );

      final log6 = '🔵 [fetchBalance] 获取余额信息';
      debugPrint(log6);
      await _writeLog(log6);
      final balanceInfo = claudeCore.getBalance(_tempSession!);
      final log7 = '🟢 [fetchBalance] 获取到余额：$balanceInfo';
      debugPrint(log7);
      await _writeLog(log7);
      state = state.copyWith(balance: balanceInfo);
    } catch (e) {
      final log8 = '🔴 [fetchBalance] 异常：$e';
      debugPrint(log8);
      await _writeLog(log8);
      state = state.copyWith(balance: {});
      rethrow;
    } finally {
      state = state.copyWith(isLoadingBalance: false);
    }
  }

  void selectModel(String model) {
    state = state.copyWith(selectedModel: model);
  }

  Future<void> saveApiKey(String provider, String apiKey) async {
    final repo = ref.read(sessionRepositoryProvider);
    final result = repo.setApiKey(provider, apiKey);
    if (result != 0) {
      throw Exception('Failed to save API Key');
    }
    updateApiKey(apiKey);
  }

  Future<String?> loadApiKey(String provider) async {
    final repo = ref.read(sessionRepositoryProvider);
    return repo.getApiKey(provider);
  }

  @override
  void dispose() {
    _destroyTempSession();
    super.dispose();
  }
}

final settingsNotifierProvider =
    StateNotifierProvider<SettingsNotifier, SettingsState>((ref) {
      return SettingsNotifier(ref);
    });
