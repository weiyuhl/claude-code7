import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';
import 'src/rust/bindings.dart';
import 'dart:ffi';
import 'dart:convert';

void main() {
  runApp(const MyApp());
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
        title: Text('Jasmine AI', style: GoogleFonts.outfit(fontWeight: FontWeight.bold)),
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
                    padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
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
              color: Theme.of(context).colorScheme.primaryContainer.withOpacity(0.3),
              shape: BoxShape.circle,
            ),
            child: Icon(Icons.auto_awesome, size: 64, color: Theme.of(context).colorScheme.primary),
          ),
          const SizedBox(height: 24),
          Text(
            '有什么可以帮您的？',
            style: GoogleFonts.outfit(fontSize: 22, fontWeight: FontWeight.w600),
          ),
          const SizedBox(height: 8),
          Text(
            '切换设置来更改模型或供应商',
            style: TextStyle(color: Theme.of(context).colorScheme.onSurfaceVariant),
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
                  contentPadding: EdgeInsets.symmetric(horizontal: 20, vertical: 12),
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
    final apiKeyController = TextEditingController(text: _apiKeys[_currentProvider]);
    
    showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(24)),
      ),
      builder: (context) => StatefulBuilder(
        builder: (context, setModalState) => Padding(
          padding: EdgeInsets.only(
            bottom: MediaQuery.of(context).viewInsets.bottom,
            left: 24, right: 24, top: 24
          ),
          child: SingleChildScrollView(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    Text('配置中心', style: GoogleFonts.outfit(fontSize: 20, fontWeight: FontWeight.bold)),
                    IconButton(
                      icon: const Icon(Icons.refresh),
                      onPressed: () {
                        _refreshInfo();
                        Navigator.pop(context);
                        _showSettings(); // Re-open to see updates
                      },
                    ),
                  ],
                ),
                if (_balance.isNotEmpty) ...[
                  const SizedBox(height: 8),
                  Container(
                    padding: const EdgeInsets.all(12),
                    decoration: BoxDecoration(
                      color: Theme.of(context).colorScheme.primaryContainer.withOpacity(0.4),
                      borderRadius: BorderRadius.circular(12),
                    ),
                    child: Row(
                      children: [
                        const Icon(Icons.account_balance_wallet_outlined, size: 20),
                        const SizedBox(width: 8),
                        Text('余额: ${_balance['total_balance'] ?? '0'} ${_balance['currency'] ?? ''}'),
                      ],
                    ),
                  ),
                ],
                const SizedBox(height: 24),
                const Text('供应商', style: TextStyle(fontWeight: FontWeight.w600)),
                const SizedBox(height: 8),
                DropdownButtonFormField<String>(
                  value: _currentProvider,
                  decoration: const InputDecoration(border: OutlineInputBorder()),
                  items: _providers.map((p) => DropdownMenuItem(value: p, child: Text(p))).toList(),
                  onChanged: (val) async {
                    if (val != null) {
                      setState(() {
                        _currentProvider = val;
                        _currentModel = '';
                        apiKeyController.text = _apiKeys[val] ?? '';
                      });
                      setModalState(() {
                        _currentProvider = val;
                        _currentModel = _models[val]?.isNotEmpty == true ? _models[val]!.first : '加载中...';
                      });
                      _createSession();
                      
                      // Fetch models after switching provider
                      if ((_apiKeys[val] ?? '').isNotEmpty) {
                        await Future.delayed(const Duration(milliseconds: 500));
                        await _refreshInfo();
                        setModalState(() {
                          _currentModel = _models[val]?.isNotEmpty == true ? _models[val]!.first : '无可用模型';
                        });
                      }
                    }
                  },
                ),
                const SizedBox(height: 16),
                const Text('API Key', style: TextStyle(fontWeight: FontWeight.w600)),
                const SizedBox(height: 8),
                TextField(
                  controller: apiKeyController,
                  obscureText: true,
                  decoration: const InputDecoration(
                    border: OutlineInputBorder(),
                    hintText: '输入您的 API Key',
                  ),
                  onChanged: (val) {
                    _apiKeys[_currentProvider] = val;
                  },
                ),
                const SizedBox(height: 16),
                const Text('模型', style: TextStyle(fontWeight: FontWeight.w600)),
                const SizedBox(height: 8),
                DropdownButtonFormField<String>(
                  value: _models[_currentProvider]?.isNotEmpty == true ? _currentModel : null,
                  decoration: const InputDecoration(border: OutlineInputBorder()),
                  hint: const Text('请先输入 API Key 获取模型列表'),
                  items: _models[_currentProvider]!.isNotEmpty
                      ? _models[_currentProvider]!.map((m) => DropdownMenuItem(value: m, child: Text(m))).toList()
                      : [],
                  onChanged: _models[_currentProvider]!.isNotEmpty ? (val) {
                    if (val != null) {
                      setModalState(() => _currentModel = val);
                      setState(() => _currentModel = val);
                      _createSession();
                    }
                  } : null,
                ),
                const SizedBox(height: 32),
              ],
            ),
          ),
        ),
      ),
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
    required this.isUser
  });

  @override
  Widget build(BuildContext context) {
    final colorScheme = Theme.of(context).colorScheme;
    
    return Padding(
      padding: const EdgeInsets.only(bottom: 16),
      child: Row(
        mainAxisAlignment: isUser ? MainAxisAlignment.end : MainAxisAlignment.start,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          if (!isUser) ...[
            CircleAvatar(
              backgroundColor: colorScheme.primaryContainer,
              child: Icon(Icons.smart_toy_outlined, size: 20, color: colorScheme.primary),
            ),
            const SizedBox(width: 12),
          ],
          Flexible(
            child: Column(
              crossAxisAlignment: isUser ? CrossAxisAlignment.end : CrossAxisAlignment.start,
              children: [
                if (!isUser && thinking != null && thinking!.isNotEmpty)
                  _buildThinkingBlock(context, thinking!),
                if (!isUser && toolUse != null)
                  _buildToolUseBlock(context, toolUse),
                if (content.isNotEmpty)
                  Container(
                    padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
                    decoration: BoxDecoration(
                      color: isUser ? colorScheme.primary : colorScheme.surfaceVariant,
                      borderRadius: BorderRadius.only(
                        topLeft: const Radius.circular(16),
                        topRight: const Radius.circular(16),
                        bottomLeft: isUser ? const Radius.circular(16) : Radius.zero,
                        bottomRight: isUser ? Radius.zero : const Radius.circular(16),
                      ),
                    ),
                    child: Text(
                      content,
                      style: TextStyle(
                        color: isUser ? colorScheme.onPrimary : colorScheme.onSurfaceVariant,
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
        border: Border.all(color: Theme.of(context).colorScheme.outlineVariant.withOpacity(0.5)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(Icons.psychology_outlined, size: 16, color: Theme.of(context).colorScheme.primary),
              const SizedBox(width: 8),
              Text('思考中...', style: GoogleFonts.outfit(fontSize: 12, fontWeight: FontWeight.w600, color: Theme.of(context).colorScheme.primary)),
            ],
          ),
          const SizedBox(height: 8),
          Text(
            text,
            style: TextStyle(
              fontSize: 14,
              fontStyle: FontStyle.italic,
              color: Theme.of(context).colorScheme.onSurfaceVariant.withOpacity(0.7),
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
              const Icon(Icons.build_circle_outlined, size: 16, color: Colors.amber),
              const SizedBox(width: 8),
              Text('调用工具', style: GoogleFonts.outfit(fontSize: 12, fontWeight: FontWeight.bold, color: Colors.amber)),
            ],
          ),
          const SizedBox(height: 4),
          Text(
            toolUse.toString(),
            style: GoogleFonts.firaCode(fontSize: 12),
          ),
        ],
      ),
    );
  }
}
