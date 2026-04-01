import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';
import 'src/rust/bindings.dart';
import 'dart:ffi';
import 'dart:convert';

void main() {
  runApp(const MyApp());
}

// 全局状态管理
class AppState {
  static final AppState _instance = AppState._internal();
  factory AppState() => _instance;
  AppState._internal();

  ClaudeCore? claudeCore;
  Pointer<Void>? session;
  String currentProvider = 'openrouter';
  String currentModel = '';
  final Map<String, String> apiKeys = {
    'openrouter': '',
    'deepseek': '',
    'siliconflow': '',
  };
  final Map<String, List<String>> models = {
    'openrouter': [],
    'deepseek': [],
    'siliconflow': [],
  };
  final List<String> providers = ['openrouter', 'deepseek', 'siliconflow'];
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      debugShowCheckedModeBanner: false,
      title: 'Jasmine AI',
      theme: ThemeData(
        useMaterial3: true,
        colorScheme: ColorScheme.fromSeed(
          seedColor: const Color(0xFF6366F1), // Indigo
          brightness: Brightness.light,
        ),
        textTheme: GoogleFonts.outfitTextTheme(),
      ),
      darkTheme: ThemeData(
        useMaterial3: true,
        colorScheme: ColorScheme.fromSeed(
          seedColor: const Color(0xFF6366F1),
          brightness: Brightness.dark,
          surface: const Color(0xFF0F172A),
        ),
        textTheme: GoogleFonts.outfitTextTheme(ThemeData.dark().textTheme),
      ),
      themeMode: ThemeMode.system,
      home: const ChatPage(),
    );
  }
}

class ChatPage extends StatefulWidget {
  const ChatPage({super.key});

  @override
  State<ChatPage> createState() => _ChatPageState();
}

class _ChatPageState extends State<ChatPage> {
  ClaudeCore? _claudeCore;
  Pointer<Void>? _session;
  final List<Map<String, dynamic>> _messages = [];
  final TextEditingController _messageController = TextEditingController();
  final ScrollController _scrollController = ScrollController();
  bool _isStreaming = false;
  String _currentProvider = 'openrouter';
  String _currentModel = '';

  // API Keys
  final Map<String, String> _apiKeys = {
    'openrouter': '',
    'deepseek': '',
    'siliconflow': '',
  };

  final List<String> _providers = ['openrouter', 'deepseek', 'siliconflow'];
  Map<String, List<String>> _models = {
    'openrouter': [],
    'deepseek': [],
    'siliconflow': [],
  };

  Map<String, dynamic> _balance = {
    'is_available': false,
    'total_balance': '0.00',
    'currency': 'USD',
  };

  @override
  void initState() {
    super.initState();
    _initRust();
  }

  void _initRust() {
    try {
      _claudeCore = ClaudeCore();
      _createSession();
    } catch (e) {
      _showError('初始化失败: $e');
    }
  }

  void _createSession() {
    if (_claudeCore == null) return;

    if (_session != null) {
      _claudeCore!.destroySession(_session!);
    }

    final config = {
      'provider': _currentProvider,
      'model': '',
      'max_tokens': 4096,
    };

    _session = _claudeCore!.createSession(config);
    _refreshInfo();
  }

  Future<void> _refreshInfo() async {
    if (_session == null || _claudeCore == null) return;

    final apiKey = _apiKeys[_currentProvider] ?? '';
    if (apiKey.isEmpty) return;

    _claudeCore!.setProvider(_session!, _currentProvider, apiKey);

    try {
      final modelsList = _claudeCore!.listModels(_session!);
      if (modelsList.isNotEmpty) {
        final modelIds = modelsList.map((m) => m['id'].toString()).toList();
        setState(() {
          _models[_currentProvider] = modelIds;
          if (_currentModel.isEmpty && modelIds.isNotEmpty) {
            _currentModel = modelIds.first;
          }
        });
      }
    } catch (e) {
      print('获取模型列表失败：$e');
    }

    try {
      final balanceInfo = _claudeCore!.getBalance(_session!);
      setState(() {
        _balance = balanceInfo;
      });
    } catch (e) {
      print('获取余额失败：$e');
    }
  }

  void _showError(String message) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(message), backgroundColor: Colors.redAccent),
    );
  }

  void _scrollToBottom() {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (_scrollController.hasClients) {
        _scrollController.animateTo(
          _scrollController.position.maxScrollExtent,
          duration: const Duration(milliseconds: 300),
          curve: Curves.easeOut,
        );
      }
    });
  }

  void _sendMessage() {
    final text = _messageController.text.trim();
    if (text.isEmpty || _isStreaming || _session == null) return;

    final apiKey = _apiKeys[_currentProvider] ?? '';
    if (apiKey.isEmpty) {
      _showError('请先在配置中心设置 API Key');
      return;
    }

    _claudeCore!.setProvider(_session!, _currentProvider, apiKey);

    setState(() {
      _messages.add({'role': 'user', 'content': text});
      _messages.add({
        'role': 'assistant',
        'content': '',
        'thinking': '',
        'tool_use': null,
      });
      _isStreaming = true;
    });
    _messageController.clear();
    _scrollToBottom();

    try {
      String fullContent = '';
      String fullThinking = '';

      _claudeCore!.streamMessage(_session!, text, (chunk) {
        if (mounted) {
          setState(() {
            final type = chunk['type'];
            final content = chunk['content'];

            if (type == 'content') {
              fullContent += content;
              _messages.last['content'] = fullContent;
            } else if (type == 'thinking') {
              fullThinking += content;
              _messages.last['thinking'] = fullThinking;
            } else if (type == 'tool_use') {
              _messages.last['tool_use'] = content;
            }
          });
          _scrollToBottom();
        }
      });

      setState(() {
        _isStreaming = false;
      });
    } catch (e) {
      _showError('发送失败: $e');
      setState(() {
        _isStreaming = false;
      });
    }
  }

  @override
  void dispose() {
    if (_session != null && _claudeCore != null) {
      _claudeCore!.destroySession(_session!);
    }
    _messageController.dispose();
    _scrollController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final colorScheme = Theme.of(context).colorScheme;

    return Scaffold(
      backgroundColor: colorScheme.surface,
      appBar: AppBar(
        title: Text(
          'Jasmine AI',
          style: GoogleFonts.outfit(fontWeight: FontWeight.bold),
        ),
        elevation: 0,
        backgroundColor: Colors.transparent,
        actions: [
          IconButton(
            icon: const Icon(Icons.settings_outlined),
            onPressed: _showSettings,
          ),
          const SizedBox(width: 8),
        ],
      ),
      body: Column(
        children: [
          Expanded(
            child: _messages.isEmpty
                ? _buildEmptyState()
                : ListView.builder(
                    controller: _scrollController,
                    padding: const EdgeInsets.symmetric(
                      horizontal: 16,
                      vertical: 8,
                    ),
                    itemCount: _messages.length,
                    itemBuilder: (context, index) {
                      final msg = _messages[index];
                      return _MessageBubble(
                        content: msg['content']?.toString() ?? '',
                        thinking: msg['thinking']?.toString(),
                        toolUse: msg['tool_use'],
                        isUser: msg['role'] == 'user',
                      );
                    },
                  ),
          ),
          _buildInputArea(colorScheme),
        ],
      ),
    );
  }

  Widget _buildEmptyState() {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Container(
            padding: const EdgeInsets.all(20),
            decoration: BoxDecoration(
              color: Theme.of(
                context,
              ).colorScheme.primaryContainer.withOpacity(0.3),
              shape: BoxShape.circle,
            ),
            child: Icon(
              Icons.auto_awesome,
              size: 64,
              color: Theme.of(context).colorScheme.primary,
            ),
          ),
          const SizedBox(height: 24),
          Text(
            '有什么可以帮您的？',
            style: GoogleFonts.outfit(
              fontSize: 22,
              fontWeight: FontWeight.w600,
            ),
          ),
          const SizedBox(height: 8),
          Text(
            '切换设置来更改模型或供应商',
            style: TextStyle(
              color: Theme.of(context).colorScheme.onSurfaceVariant,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildInputArea(ColorScheme colorScheme) {
    return Container(
      padding: const EdgeInsets.fromLTRB(16, 8, 16, 24),
      decoration: BoxDecoration(
        color: colorScheme.surface,
        boxShadow: [
          BoxShadow(
            color: Colors.black.withOpacity(0.05),
            blurRadius: 10,
            offset: const Offset(0, -5),
          ),
        ],
      ),
      child: Row(
        children: [
          Expanded(
            child: Container(
              decoration: BoxDecoration(
                color: colorScheme.surfaceVariant.withOpacity(0.3),
                borderRadius: BorderRadius.circular(24),
                border: Border.all(color: colorScheme.outlineVariant),
              ),
              child: TextField(
                controller: _messageController,
                maxLines: 5,
                minLines: 1,
                decoration: const InputDecoration(
                  hintText: '输入消息...',
                  contentPadding: EdgeInsets.symmetric(
                    horizontal: 20,
                    vertical: 12,
                  ),
                  border: InputBorder.none,
                ),
              ),
            ),
          ),
          const SizedBox(width: 8),
          GestureDetector(
            onTap: _isStreaming ? null : _sendMessage,
            child: Container(
              width: 48,
              height: 48,
              decoration: BoxDecoration(
                color: _isStreaming ? colorScheme.outline : colorScheme.primary,
                shape: BoxShape.circle,
              ),
              child: Icon(
                _isStreaming ? Icons.hourglass_empty : Icons.send_rounded,
                color: colorScheme.onPrimary,
                size: 20,
              ),
            ),
          ),
        ],
      ),
    );
  }

  void _showSettings() {
    Navigator.push(
      context,
      MaterialPageRoute(
        builder: (context) => SettingsPage(
          claudeCore: _claudeCore,
          currentProvider: _currentProvider,
          currentModel: _currentModel,
          apiKeys: _apiKeys,
          models: _models,
          providers: _providers,
          onSettingsChanged: (provider, model) {
            setState(() {
              _currentProvider = provider;
              _currentModel = model;
            });
            _createSession();
          },
        ),
      ),
    );
  }
}

class SettingsPage extends StatefulWidget {
  final ClaudeCore? claudeCore;
  final String currentProvider;
  final String currentModel;
  final Map<String, String> apiKeys;
  final Map<String, List<String>> models;
  final List<String> providers;
  final Function(String provider, String model) onSettingsChanged;

  const SettingsPage({
    super.key,
    required this.claudeCore,
    required this.currentProvider,
    required this.currentModel,
    required this.apiKeys,
    required this.models,
    required this.providers,
    required this.onSettingsChanged,
  });

  @override
  State<SettingsPage> createState() => _SettingsPageState();
}

class _SettingsPageState extends State<SettingsPage> {
  late String _selectedProvider;
  late String _selectedModel;
  late TextEditingController _apiKeyController;
  late Map<String, List<String>> _models;
  bool _isLoadingModels = false;
  bool _isLoadingBalance = false;
  Map<String, dynamic> _balance = {};
  Pointer<Void>? _tempSession;

  @override
  void initState() {
    super.initState();
    _selectedProvider = widget.currentProvider;
    _selectedModel = widget.currentModel;
    _apiKeyController = TextEditingController(
      text: widget.apiKeys[_selectedProvider] ?? '',
    );
    _models = Map.from(widget.models);
    _createTempSession();
  }

  @override
  void dispose() {
    _apiKeyController.dispose();
    _destroyTempSession();
    super.dispose();
  }

  void _createTempSession() {
    if (widget.claudeCore == null) return;

    _destroyTempSession();

    final config = {
      'provider': _selectedProvider,
      'model': 'auto',
      'max_tokens': 4096,
    };

    _tempSession = widget.claudeCore!.createSession(config);
    if (_tempSession == nullptr) {
      _showSnackBar('创建会话失败', isError: true);
    }
  }

  void _destroyTempSession() {
    if (_tempSession != null && widget.claudeCore != null) {
      widget.claudeCore!.destroySession(_tempSession!);
      _tempSession = null;
    }
  }

  Future<void> _fetchModels() async {
    if (widget.claudeCore == null) {
      _showSnackBar('Rust 核心未初始化', isError: true);
      return;
    }

    if (_tempSession == null || _tempSession == nullptr) {
      _showSnackBar('会话未创建，请重试', isError: true);
      return;
    }

    final apiKey = _apiKeyController.text.trim();
    if (apiKey.isEmpty) {
      _showSnackBar('请先输入 API Key', isError: true);
      return;
    }

    setState(() {
      _isLoadingModels = true;
    });

    try {
      final providerSet = widget.claudeCore!.setProvider(
        _tempSession!,
        _selectedProvider,
        apiKey,
      );
      if (!providerSet) {
        _showSnackBar('设置供应商失败', isError: true);
        return;
      }

      final modelsList = widget.claudeCore!.listModels(_tempSession!);

      if (modelsList.isNotEmpty) {
        final modelIds = modelsList.map((m) => m['id'].toString()).toList();
        setState(() {
          _models[_selectedProvider] = modelIds;
          if (modelIds.isNotEmpty && !modelIds.contains(_selectedModel)) {
            _selectedModel = modelIds.first;
          }
        });
        _showSnackBar('成功获取 ${modelIds.length} 个模型');
      } else {
        _showSnackBar('未获取到模型列表', isError: true);
      }
    } catch (e) {
      _showSnackBar('获取模型失败: $e', isError: true);
    } finally {
      setState(() {
        _isLoadingModels = false;
      });
    }
  }

  Future<void> _fetchBalance() async {
    if (widget.claudeCore == null) {
      _showSnackBar('Rust 核心未初始化', isError: true);
      return;
    }

    if (_tempSession == null || _tempSession == nullptr) {
      _showSnackBar('会话未创建，请重试', isError: true);
      return;
    }

    final apiKey = _apiKeyController.text.trim();
    if (apiKey.isEmpty) {
      _showSnackBar('请先输入 API Key', isError: true);
      return;
    }

    setState(() {
      _isLoadingBalance = true;
    });

    try {
      final providerSet = widget.claudeCore!.setProvider(
        _tempSession!,
        _selectedProvider,
        apiKey,
      );
      if (!providerSet) {
        _showSnackBar('设置供应商失败', isError: true);
        return;
      }

      final balanceInfo = widget.claudeCore!.getBalance(_tempSession!);

      setState(() {
        _balance = balanceInfo;
      });

      final balance = balanceInfo['total_balance'] ?? '0';
      final currency = balanceInfo['currency'] ?? '';
      _showSnackBar('余额: $balance $currency');
    } catch (e) {
      _showSnackBar('获取余额失败: $e', isError: true);
    } finally {
      setState(() {
        _isLoadingBalance = false;
      });
    }
  }

  void _saveSettings() {
    final apiKey = _apiKeyController.text.trim();
    if (apiKey.isEmpty) {
      _showSnackBar('请输入 API Key', isError: true);
      return;
    }

    // 保存 API Key
    widget.apiKeys[_selectedProvider] = apiKey;

    // 回调通知父页面
    widget.onSettingsChanged(_selectedProvider, _selectedModel);

    _showSnackBar('设置已保存');
    Navigator.pop(context);
  }

  void _showSnackBar(String message, {bool isError = false}) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(message),
        backgroundColor: isError ? Colors.redAccent : Colors.green,
        duration: const Duration(seconds: 2),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final colorScheme = Theme.of(context).colorScheme;
    final currentModels = _models[_selectedProvider] ?? [];

    return Scaffold(
      appBar: AppBar(
        title: Text(
          '供应商配置',
          style: GoogleFonts.outfit(fontWeight: FontWeight.bold),
        ),
        elevation: 0,
        backgroundColor: Colors.transparent,
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // 供应商选择
            _buildSectionTitle('供应商'),
            const SizedBox(height: 8),
            DropdownButtonFormField<String>(
              value: _selectedProvider,
              decoration: const InputDecoration(
                border: OutlineInputBorder(),
                prefixIcon: Icon(Icons.cloud_outlined),
              ),
              items: widget.providers.map((p) {
                return DropdownMenuItem(value: p, child: Text(p.toUpperCase()));
              }).toList(),
              onChanged: (val) {
                if (val != null) {
                  setState(() {
                    _selectedProvider = val;
                    _apiKeyController.text = widget.apiKeys[val] ?? '';
                    _selectedModel = '';
                  });
                }
              },
            ),

            const SizedBox(height: 24),

            // API Key 输入
            _buildSectionTitle('API Key'),
            const SizedBox(height: 8),
            TextField(
              controller: _apiKeyController,
              obscureText: true,
              decoration: const InputDecoration(
                border: OutlineInputBorder(),
                prefixIcon: Icon(Icons.key_outlined),
                hintText: '输入您的 API Key',
              ),
            ),

            const SizedBox(height: 24),

            // 获取模型列表按钮
            SizedBox(
              width: double.infinity,
              child: OutlinedButton.icon(
                onPressed: _isLoadingModels ? null : _fetchModels,
                icon: _isLoadingModels
                    ? const SizedBox(
                        width: 16,
                        height: 16,
                        child: CircularProgressIndicator(strokeWidth: 2),
                      )
                    : const Icon(Icons.refresh),
                label: Text(_isLoadingModels ? '获取中...' : '获取模型列表'),
                style: OutlinedButton.styleFrom(
                  padding: const EdgeInsets.symmetric(vertical: 12),
                ),
              ),
            ),

            const SizedBox(height: 16),

            // 模型选择
            _buildSectionTitle('模型'),
            const SizedBox(height: 8),
            DropdownButtonFormField<String>(
              value: currentModels.isNotEmpty ? _selectedModel : null,
              decoration: const InputDecoration(
                border: OutlineInputBorder(),
                prefixIcon: Icon(Icons.model_training_outlined),
                hintText: '请先获取模型列表',
              ),
              items: currentModels.map((m) {
                return DropdownMenuItem(
                  value: m,
                  child: Text(m, overflow: TextOverflow.ellipsis),
                );
              }).toList(),
              onChanged: currentModels.isNotEmpty
                  ? (val) {
                      if (val != null) {
                        setState(() {
                          _selectedModel = val;
                        });
                      }
                    }
                  : null,
            ),

            const SizedBox(height: 24),

            // 获取余额按钮
            SizedBox(
              width: double.infinity,
              child: OutlinedButton.icon(
                onPressed: _isLoadingBalance ? null : _fetchBalance,
                icon: _isLoadingBalance
                    ? const SizedBox(
                        width: 16,
                        height: 16,
                        child: CircularProgressIndicator(strokeWidth: 2),
                      )
                    : const Icon(Icons.account_balance_wallet_outlined),
                label: Text(_isLoadingBalance ? '获取中...' : '获取余额'),
                style: OutlinedButton.styleFrom(
                  padding: const EdgeInsets.symmetric(vertical: 12),
                ),
              ),
            ),

            const SizedBox(height: 16),

            // 余额显示
            if (_balance.isNotEmpty)
              Container(
                width: double.infinity,
                padding: const EdgeInsets.all(16),
                decoration: BoxDecoration(
                  color: colorScheme.primaryContainer.withOpacity(0.4),
                  borderRadius: BorderRadius.circular(12),
                ),
                child: Row(
                  children: [
                    const Icon(Icons.account_balance_wallet_outlined, size: 24),
                    const SizedBox(width: 12),
                    Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          '当前余额',
                          style: TextStyle(
                            fontSize: 12,
                            color: colorScheme.onSurfaceVariant,
                          ),
                        ),
                        Text(
                          '${_balance['total_balance'] ?? '0'} ${_balance['currency'] ?? ''}',
                          style: const TextStyle(
                            fontSize: 20,
                            fontWeight: FontWeight.bold,
                          ),
                        ),
                      ],
                    ),
                  ],
                ),
              ),

            const SizedBox(height: 32),

            // 保存按钮
            SizedBox(
              width: double.infinity,
              height: 48,
              child: ElevatedButton.icon(
                onPressed: _saveSettings,
                icon: const Icon(Icons.save_outlined),
                label: const Text('保存设置'),
                style: ElevatedButton.styleFrom(
                  backgroundColor: colorScheme.primary,
                  foregroundColor: colorScheme.onPrimary,
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildSectionTitle(String title) {
    return Text(
      title,
      style: const TextStyle(fontSize: 14, fontWeight: FontWeight.w600),
    );
  }
}

class _MessageBubble extends StatelessWidget {
  final String content;
  final String? thinking;
  final dynamic toolUse;
  final bool isUser;

  const _MessageBubble({
    required this.content,
    this.thinking,
    this.toolUse,
    required this.isUser,
  });

  @override
  Widget build(BuildContext context) {
    final colorScheme = Theme.of(context).colorScheme;

    return Padding(
      padding: const EdgeInsets.only(bottom: 16),
      child: Row(
        mainAxisAlignment: isUser
            ? MainAxisAlignment.end
            : MainAxisAlignment.start,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          if (!isUser) ...[
            CircleAvatar(
              backgroundColor: colorScheme.primaryContainer,
              child: Icon(
                Icons.smart_toy_outlined,
                size: 20,
                color: colorScheme.primary,
              ),
            ),
            const SizedBox(width: 12),
          ],
          Flexible(
            child: Column(
              crossAxisAlignment: isUser
                  ? CrossAxisAlignment.end
                  : CrossAxisAlignment.start,
              children: [
                if (!isUser && thinking != null && thinking!.isNotEmpty)
                  _buildThinkingBlock(context, thinking!),
                if (!isUser && toolUse != null)
                  _buildToolUseBlock(context, toolUse),
                if (content.isNotEmpty)
                  Container(
                    padding: const EdgeInsets.symmetric(
                      horizontal: 16,
                      vertical: 12,
                    ),
                    decoration: BoxDecoration(
                      color: isUser
                          ? colorScheme.primary
                          : colorScheme.surfaceVariant,
                      borderRadius: BorderRadius.only(
                        topLeft: const Radius.circular(16),
                        topRight: const Radius.circular(16),
                        bottomLeft: isUser
                            ? const Radius.circular(16)
                            : Radius.zero,
                        bottomRight: isUser
                            ? Radius.zero
                            : const Radius.circular(16),
                      ),
                    ),
                    child: Text(
                      content,
                      style: TextStyle(
                        color: isUser
                            ? colorScheme.onPrimary
                            : colorScheme.onSurfaceVariant,
                        fontSize: 16,
                        height: 1.4,
                      ),
                    ),
                  ),
              ],
            ),
          ),
          if (isUser) const SizedBox(width: 44),
        ],
      ),
    );
  }

  Widget _buildThinkingBlock(BuildContext context, String text) {
    return Container(
      margin: const EdgeInsets.only(bottom: 8),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceVariant.withOpacity(0.4),
        borderRadius: BorderRadius.circular(12),
        border: Border.all(
          color: Theme.of(context).colorScheme.outlineVariant.withOpacity(0.5),
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(
                Icons.psychology_outlined,
                size: 16,
                color: Theme.of(context).colorScheme.primary,
              ),
              const SizedBox(width: 8),
              Text(
                '思考中...',
                style: GoogleFonts.outfit(
                  fontSize: 12,
                  fontWeight: FontWeight.w600,
                  color: Theme.of(context).colorScheme.primary,
                ),
              ),
            ],
          ),
          const SizedBox(height: 8),
          Text(
            text,
            style: TextStyle(
              fontSize: 14,
              fontStyle: FontStyle.italic,
              color: Theme.of(
                context,
              ).colorScheme.onSurfaceVariant.withOpacity(0.7),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildToolUseBlock(BuildContext context, dynamic toolUse) {
    return Container(
      margin: const EdgeInsets.only(bottom: 8),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Colors.amber.withOpacity(0.1),
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: Colors.amber.withOpacity(0.3)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              const Icon(
                Icons.build_circle_outlined,
                size: 16,
                color: Colors.amber,
              ),
              const SizedBox(width: 8),
              Text(
                '调用工具',
                style: GoogleFonts.outfit(
                  fontSize: 12,
                  fontWeight: FontWeight.bold,
                  color: Colors.amber,
                ),
              ),
            ],
          ),
          const SizedBox(height: 4),
          Text(toolUse.toString(), style: GoogleFonts.firaCode(fontSize: 12)),
        ],
      ),
    );
  }
}
