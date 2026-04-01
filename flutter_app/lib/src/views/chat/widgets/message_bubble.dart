import 'package:flutter/material.dart';
import '../../../core/app_theme.dart';

class MessageBubble extends StatelessWidget {
  final String content;
  final String? thinking;
  final dynamic toolUse;
  final bool isUser;

  const MessageBubble({
    super.key,
    required this.content,
    this.thinking,
    this.toolUse,
    required this.isUser,
  });

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;

    return Padding(
      padding: const EdgeInsets.only(bottom: 12),
      child: Row(
        mainAxisAlignment: isUser
            ? MainAxisAlignment.end
            : MainAxisAlignment.start,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          if (!isUser) ...[
            const CircleAvatar(
              radius: 16,
              backgroundColor: AppTheme.primaryColor,
              child: Icon(Icons.auto_awesome, size: 16, color: Colors.white),
            ),
            const SizedBox(width: 8),
          ],
          Flexible(
            child: Column(
              crossAxisAlignment: isUser
                  ? CrossAxisAlignment.end
                  : CrossAxisAlignment.start,
              children: [
                if (!isUser && thinking != null && thinking!.isNotEmpty)
                  ThinkingBlock(text: thinking!),
                if (!isUser && toolUse != null) ToolUseBlock(toolUse: toolUse),
                if (content.isNotEmpty)
                  Container(
                    padding: const EdgeInsets.symmetric(
                      horizontal: 14,
                      vertical: 10,
                    ),
                    decoration: BoxDecoration(
                      color: isUser
                          ? AppTheme.primaryColor
                          : (isDark ? AppTheme.darkCard : AppTheme.lightCard),
                      borderRadius: BorderRadius.only(
                        topLeft: const Radius.circular(18),
                        topRight: const Radius.circular(18),
                        bottomLeft: isUser
                            ? const Radius.circular(18)
                            : Radius.zero,
                        bottomRight: isUser
                            ? Radius.zero
                            : const Radius.circular(18),
                      ),
                    ),
                    child: Text(
                      content,
                      style: TextStyle(
                        color: isUser ? Colors.white : null,
                        fontSize: 16,
                        height: 1.45,
                      ),
                    ),
                  ),
              ],
            ),
          ),
          if (isUser) const SizedBox(width: 32),
        ],
      ),
    );
  }
}

class ThinkingBlock extends StatelessWidget {
  final String text;

  const ThinkingBlock({super.key, required this.text});

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;

    return Container(
      margin: const EdgeInsets.only(bottom: 8),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: isDark ? AppTheme.darkCard : AppTheme.lightSurface,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(
          color: isDark ? AppTheme.darkBorder : AppTheme.lightBorderLight,
          width: 0.5,
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              const Icon(
                Icons.psychology_outlined,
                size: 14,
                color: AppTheme.secondaryColor,
              ),
              const SizedBox(width: 6),
              const Text(
                '思考中...',
                style: TextStyle(
                  fontSize: 12,
                  fontWeight: FontWeight.w500,
                  color: AppTheme.secondaryColor,
                ),
              ),
            ],
          ),
          const SizedBox(height: 6),
          Text(
            text,
            style: TextStyle(
              fontSize: 14,
              fontStyle: FontStyle.italic,
              color: isDark
                  ? AppTheme.darkTextSecondary
                  : AppTheme.lightTextSecondary,
            ),
          ),
        ],
      ),
    );
  }
}

class ToolUseBlock extends StatelessWidget {
  final dynamic toolUse;

  const ToolUseBlock({super.key, required this.toolUse});

  @override
  Widget build(BuildContext context) {
    return Container(
      margin: const EdgeInsets.only(bottom: 8),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: AppTheme.warningColor.withOpacity(0.1),
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: AppTheme.warningColor.withOpacity(0.3)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              const Icon(
                Icons.build_circle_outlined,
                size: 14,
                color: AppTheme.warningColor,
              ),
              const SizedBox(width: 6),
              const Text(
                '调用工具',
                style: TextStyle(
                  fontSize: 12,
                  fontWeight: FontWeight.w500,
                  color: AppTheme.warningColor,
                ),
              ),
            ],
          ),
          const SizedBox(height: 4),
          Text(toolUse.toString(), style: const TextStyle(fontSize: 12)),
        ],
      ),
    );
  }
}
