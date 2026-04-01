import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../core/app_theme.dart';
import '../../core/app_config.dart';
import '../../viewmodels/viewmodels.dart';

class SettingsPage extends ConsumerStatefulWidget {
  const SettingsPage({super.key});

  @override
  ConsumerState<SettingsPage> createState() => _SettingsPageState();
}

class _SettingsPageState extends ConsumerState<SettingsPage> {
  late TextEditingController _apiKeyController;

  @override
  void initState() {
    super.initState();
    final state = ref.read(settingsNotifierProvider);
    _apiKeyController = TextEditingController(text: state.apiKey);
  }

  @override
  void dispose() {
    _apiKeyController.dispose();
    ref.read(settingsNotifierProvider.notifier).dispose();
    super.dispose();
  }

  void _fetchModels() async {
    ref
        .read(settingsNotifierProvider.notifier)
        .updateApiKey(_apiKeyController.text.trim());
    try {
      await ref.read(settingsNotifierProvider.notifier).fetchModels();
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: const Text('获取模型成功'),
            backgroundColor: AppTheme.successColor,
            behavior: SnackBarBehavior.floating,
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(12),
            ),
          ),
        );
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('获取模型失败: $e'),
            backgroundColor: AppTheme.dangerColor,
            behavior: SnackBarBehavior.floating,
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(12),
            ),
          ),
        );
      }
    }
  }

  void _fetchBalance() async {
    ref
        .read(settingsNotifierProvider.notifier)
        .updateApiKey(_apiKeyController.text.trim());
    try {
      await ref.read(settingsNotifierProvider.notifier).fetchBalance();
      final state = ref.read(settingsNotifierProvider);
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(
              '余额: ${state.balance['total_balance'] ?? '0'} ${state.balance['currency'] ?? ''}',
            ),
            backgroundColor: AppTheme.successColor,
            behavior: SnackBarBehavior.floating,
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(12),
            ),
          ),
        );
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('获取余额失败: $e'),
            backgroundColor: AppTheme.dangerColor,
            behavior: SnackBarBehavior.floating,
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(12),
            ),
          ),
        );
      }
    }
  }

  void _saveSettings() {
    final apiKey = _apiKeyController.text.trim();
    if (apiKey.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: const Text('请输入 API Key'),
          backgroundColor: AppTheme.dangerColor,
          behavior: SnackBarBehavior.floating,
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(12),
          ),
        ),
      );
      return;
    }

    final state = ref.read(settingsNotifierProvider);
    final chatNotifier = ref.read(chatNotifierProvider.notifier);
    chatNotifier.updateApiKey(state.selectedProvider, apiKey);
    chatNotifier.updateProvider(state.selectedProvider);

    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: const Text('设置已保存'),
        backgroundColor: AppTheme.successColor,
        behavior: SnackBarBehavior.floating,
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
      ),
    );
    Navigator.pop(context);
  }

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(settingsNotifierProvider);
    final isDark = Theme.of(context).brightness == Brightness.dark;

    return Scaffold(
      backgroundColor: isDark ? AppTheme.darkSurface : AppTheme.lightSurface,
      appBar: AppBar(
        title: const Text('供应商配置'),
        backgroundColor: isDark ? AppTheme.darkSurface : AppTheme.lightSurface,
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            _buildSectionTitle('供应商'),
            const SizedBox(height: 8),
            _buildDropdown(
              value: state.selectedProvider,
              items: AppConfig.providers.map((p) {
                return DropdownMenuItem(value: p, child: Text(p.toUpperCase()));
              }).toList(),
              onChanged: (val) {
                if (val != null) {
                  ref
                      .read(settingsNotifierProvider.notifier)
                      .updateProvider(val, _apiKeyController.text.trim());
                }
              },
            ),
            const SizedBox(height: 24),
            _buildSectionTitle('API Key'),
            const SizedBox(height: 8),
            _buildTextField(
              controller: _apiKeyController,
              hintText: '输入您的 API Key',
              obscureText: true,
            ),
            const SizedBox(height: 24),
            SizedBox(
              width: double.infinity,
              height: 48,
              child: ElevatedButton.icon(
                onPressed: state.isLoadingModels ? null : _fetchModels,
                icon: state.isLoadingModels
                    ? const SizedBox(
                        width: 16,
                        height: 16,
                        child: CircularProgressIndicator(
                          strokeWidth: 2,
                          color: Colors.white,
                        ),
                      )
                    : const Icon(Icons.refresh, size: 18),
                label: Text(state.isLoadingModels ? '获取中...' : '获取模型列表'),
                style: ElevatedButton.styleFrom(
                  backgroundColor: AppTheme.primaryColor,
                  foregroundColor: Colors.white,
                  shape: RoundedRectangleBorder(
                    borderRadius: BorderRadius.circular(12),
                  ),
                ),
              ),
            ),
            const SizedBox(height: 16),
            _buildSectionTitle('模型'),
            const SizedBox(height: 8),
            _buildDropdown(
              value: state.models.isNotEmpty ? state.selectedModel : null,
              items: state.models.map((m) {
                return DropdownMenuItem(
                  value: m,
                  child: Text(m, overflow: TextOverflow.ellipsis),
                );
              }).toList(),
              onChanged: state.models.isNotEmpty
                  ? (val) {
                      if (val != null) {
                        ref
                            .read(settingsNotifierProvider.notifier)
                            .selectModel(val);
                      }
                    }
                  : null,
              hintText: '请先获取模型列表',
            ),
            const SizedBox(height: 24),
            SizedBox(
              width: double.infinity,
              height: 48,
              child: ElevatedButton.icon(
                onPressed: state.isLoadingBalance ? null : _fetchBalance,
                icon: state.isLoadingBalance
                    ? const SizedBox(
                        width: 16,
                        height: 16,
                        child: CircularProgressIndicator(
                          strokeWidth: 2,
                          color: Colors.white,
                        ),
                      )
                    : const Icon(Icons.account_balance_wallet, size: 18),
                label: Text(state.isLoadingBalance ? '获取中...' : '获取余额'),
                style: ElevatedButton.styleFrom(
                  backgroundColor: AppTheme.primaryColor,
                  foregroundColor: Colors.white,
                  shape: RoundedRectangleBorder(
                    borderRadius: BorderRadius.circular(12),
                  ),
                ),
              ),
            ),
            const SizedBox(height: 16),
            if (state.balance.isNotEmpty)
              Container(
                width: double.infinity,
                padding: const EdgeInsets.all(16),
                decoration: BoxDecoration(
                  color: isDark ? AppTheme.darkCard : AppTheme.lightCard,
                  borderRadius: BorderRadius.circular(12),
                  border: Border.all(
                    color: isDark
                        ? AppTheme.darkBorder
                        : AppTheme.lightBorderLight,
                    width: 0.5,
                  ),
                ),
                child: Row(
                  children: [
                    const Icon(
                      Icons.account_balance_wallet,
                      size: 20,
                      color: AppTheme.primaryColor,
                    ),
                    const SizedBox(width: 12),
                    Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          '当前余额',
                          style: TextStyle(
                            fontSize: 13,
                            color: isDark
                                ? AppTheme.darkTextSecondary
                                : AppTheme.lightTextSecondary,
                          ),
                        ),
                        Text(
                          '${state.balance['total_balance'] ?? '0'} ${state.balance['currency'] ?? ''}',
                          style: const TextStyle(
                            fontSize: 18,
                            fontWeight: FontWeight.w600,
                          ),
                        ),
                      ],
                    ),
                  ],
                ),
              ),
            const SizedBox(height: 32),
            SizedBox(
              width: double.infinity,
              height: 48,
              child: ElevatedButton.icon(
                onPressed: _saveSettings,
                icon: const Icon(Icons.save, size: 18),
                label: const Text('保存设置'),
                style: ElevatedButton.styleFrom(
                  backgroundColor: AppTheme.successColor,
                  foregroundColor: Colors.white,
                  shape: RoundedRectangleBorder(
                    borderRadius: BorderRadius.circular(12),
                  ),
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
      style: const TextStyle(
        fontSize: 15,
        fontWeight: FontWeight.w500,
        color: AppTheme.lightTextSecondary,
      ),
    );
  }

  Widget _buildTextField({
    required TextEditingController controller,
    String? hintText,
    bool obscureText = false,
  }) {
    final isDark = Theme.of(context).brightness == Brightness.dark;

    return Container(
      decoration: BoxDecoration(
        color: isDark ? AppTheme.darkCard : AppTheme.lightCard,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(
          color: isDark ? AppTheme.darkBorder : AppTheme.lightBorder,
          width: 0.5,
        ),
      ),
      child: TextField(
        controller: controller,
        obscureText: obscureText,
        style: TextStyle(
          fontSize: 16,
          color: isDark ? AppTheme.darkText : AppTheme.lightText,
        ),
        decoration: InputDecoration(
          hintText: hintText,
          hintStyle: const TextStyle(
            color: AppTheme.lightTextSecondary,
            fontSize: 16,
          ),
          contentPadding: const EdgeInsets.symmetric(
            horizontal: 16,
            vertical: 14,
          ),
          border: InputBorder.none,
        ),
      ),
    );
  }

  Widget _buildDropdown({
    required String? value,
    required List<DropdownMenuItem<String>> items,
    void Function(String?)? onChanged,
    String? hintText,
  }) {
    final isDark = Theme.of(context).brightness == Brightness.dark;

    return Container(
      decoration: BoxDecoration(
        color: isDark ? AppTheme.darkCard : AppTheme.lightCard,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(
          color: isDark ? AppTheme.darkBorder : AppTheme.lightBorder,
          width: 0.5,
        ),
      ),
      child: DropdownButtonHideUnderline(
        child: DropdownButtonFormField<String>(
          value: value,
          isExpanded: true,
          decoration: InputDecoration(
            hintText: hintText,
            hintStyle: const TextStyle(
              color: AppTheme.lightTextSecondary,
              fontSize: 16,
            ),
            contentPadding: const EdgeInsets.symmetric(
              horizontal: 16,
              vertical: 4,
            ),
            border: InputBorder.none,
          ),
          items: items,
          onChanged: onChanged,
          style: TextStyle(
            fontSize: 16,
            color: isDark ? AppTheme.darkText : AppTheme.lightText,
          ),
          dropdownColor: isDark ? AppTheme.darkCard : AppTheme.lightCard,
        ),
      ),
    );
  }
}
