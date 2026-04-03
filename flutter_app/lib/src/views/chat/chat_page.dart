import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../core/app_theme.dart';
import '../../core/app_config.dart';
import '../../viewmodels/viewmodels.dart';
import '../../viewmodels/providers.dart';
import '../settings/settings_page.dart';
import 'widgets/widgets.dart';

class ChatPage extends ConsumerStatefulWidget {
  const ChatPage({super.key});

  @override
  ConsumerState<ChatPage> createState() => _ChatPageState();
}

class _ChatPageState extends ConsumerState<ChatPage> {
  final TextEditingController _messageController = TextEditingController();
  final ScrollController _scrollController = ScrollController();

  @override
  void dispose() {
    _messageController.dispose();
    _scrollController.dispose();
    super.dispose();
  }

  void _sendMessage() {
    final text = _messageController.text.trim();
    _writeLogSync('🔵 [_sendMessage] 被调用，text: "$text"');
    
    if (text.isEmpty) {
      _writeLogSync('🔴 [_sendMessage] 消息为空，返回');
      return;
    }

    final state = ref.read(chatNotifierProvider);
    _writeLogSync('🔵 [_sendMessage] isStreaming: ${state.isStreaming}');
    
    if (state.isStreaming) {
      _writeLogSync('🔴 [_sendMessage] 正在流式传输，返回');
      return;
    }

    final repo = ref.read(sessionRepositoryProvider);
    final apiKey = repo.getApiKey(state.currentProvider) ?? state.apiKeys[state.currentProvider] ?? '';
    
    _writeLogSync('🔵 [_sendMessage] Provider: ${state.currentProvider}, API Key: ${apiKey.isEmpty ? "空" : "有值"}');
    
    if (apiKey.isEmpty) {
      _writeLogSync('🔴 [_sendMessage] API Key 为空，显示错误');
      _showError('请先在设置中配置 API Key');
      return;
    }

    _messageController.clear();
    _scrollToBottom();

    try {
      _writeLogSync('🔵 [_sendMessage] 准备发送消息');
      ref.read(chatNotifierProvider.notifier).sendMessage(text);
      _writeLogSync('🔵 [_sendMessage] sendMessage 调用完成');
    } catch (e) {
      _writeLogSync('🔴 [_sendMessage] 发送异常：$e');
      _showError('发送失败：$e');
    }
  }

  void _writeLogSync(String message) {
    try {
      final settingsNotifier = ref.read(settingsNotifierProvider.notifier);
      // 同步写入日志
      settingsNotifier.writeLogForChat(message);
    } catch (e) {
      // 忽略错误
    }
  }

  void _writeLog(String message) {
    final settingsNotifier = ref.read(settingsNotifierProvider.notifier);
    settingsNotifier.writeLogForChat(message);
  }

  void _showError(String message) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(message),
        backgroundColor: AppTheme.dangerColor,
        behavior: SnackBarBehavior.floating,
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
      ),
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

  void _showSettings() {
    Navigator.push(
      context,
      MaterialPageRoute(builder: (context) => const SettingsPage()),
    );
  }

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(chatNotifierProvider);
    final isDark = Theme.of(context).brightness == Brightness.dark;

    return Scaffold(
      backgroundColor: isDark ? AppTheme.darkSurface : AppTheme.lightSurface,
      appBar: AppBar(
        title: const Text(AppConfig.appName),
        actions: [
          IconButton(
            icon: const Icon(Icons.settings, size: 22),
            onPressed: _showSettings,
          ),
          const SizedBox(width: 8),
        ],
      ),
      body: Column(
        children: [
          Expanded(
            child: state.messages.isEmpty
                ? _buildEmptyState()
                : ListView.builder(
                    controller: _scrollController,
                    padding: const EdgeInsets.symmetric(
                      horizontal: 16,
                      vertical: 8,
                    ),
                    itemCount: state.messages.length,
                    itemBuilder: (context, index) {
                      final msg = state.messages[index];
                      return MessageBubble(
                        content: msg['content']?.toString() ?? '',
                        thinking: msg['thinking']?.toString(),
                        toolUse: msg['tool_use'],
                        isUser: msg['role'] == 'user',
                      );
                    },
                  ),
          ),
          _buildInputArea(state.isStreaming),
        ],
      ),
    );
  }

  Widget _buildEmptyState() {
    final isDark = Theme.of(context).brightness == Brightness.dark;

    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(
            Icons.chat_bubble_outline,
            size: 72,
            color: isDark
                ? AppTheme.darkTextSecondary
                : AppTheme.lightTextSecondary,
          ),
          const SizedBox(height: 16),
          const Text(
            '有什么可以帮您的？',
            style: TextStyle(
              fontSize: 22,
              fontWeight: FontWeight.w600,
              letterSpacing: -0.3,
            ),
          ),
          const SizedBox(height: 8),
          const Text(
            '点击右上角设置来配置 API',
            style: TextStyle(fontSize: 15, color: AppTheme.lightTextSecondary),
          ),
        ],
      ),
    );
  }

  Widget _buildInputArea(bool isStreaming) {
    final isDark = Theme.of(context).brightness == Brightness.dark;

    return Container(
      padding: const EdgeInsets.fromLTRB(16, 8, 16, 20),
      color: isDark ? AppTheme.darkSurface : AppTheme.lightSurface,
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.end,
        children: [
          Expanded(
            child: Container(
              decoration: BoxDecoration(
                color: isDark ? AppTheme.darkCard : AppTheme.lightCard,
                borderRadius: BorderRadius.circular(20),
                border: Border.all(
                  color: isDark ? AppTheme.darkBorder : AppTheme.lightBorder,
                  width: 0.5,
                ),
              ),
              child: TextField(
                controller: _messageController,
                maxLines: 5,
                minLines: 1,
                style: const TextStyle(fontSize: 16),
                decoration: const InputDecoration(
                  hintText: '输入消息...',
                  hintStyle: TextStyle(
                    color: AppTheme.lightTextSecondary,
                    fontSize: 16,
                  ),
                  contentPadding: EdgeInsets.symmetric(
                    horizontal: 16,
                    vertical: 10,
                  ),
                  border: InputBorder.none,
                ),
              ),
            ),
          ),
          const SizedBox(width: 8),
          // 发送按钮
          GestureDetector(
            onTap: () {
              _writeLog('🔵 [发送按钮] 被点击！');
              if (isStreaming) {
                _writeLog('🔵 [发送按钮] 正在流式传输中，忽略点击');
                return;
              }
              _sendMessage();
            },
            child: Container(
              width: 40,
              height: 40,
              decoration: BoxDecoration(
                color: isStreaming
                    ? (isDark ? AppTheme.darkBorder : AppTheme.lightBorder)
                    : AppTheme.primaryColor,
                shape: BoxShape.circle,
              ),
              child: Icon(
                isStreaming ? Icons.hourglass_empty : Icons.arrow_upward,
                color: isDark ? AppTheme.darkTextSecondary : Colors.white,
                size: 18,
              ),
            ),
          ),
        ],
      ),
    );
  }
}
