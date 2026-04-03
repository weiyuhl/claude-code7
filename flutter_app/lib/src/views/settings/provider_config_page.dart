import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../core/app_theme.dart';
import '../../core/app_config.dart';
import '../../viewmodels/viewmodels.dart';

class ProviderConfigPage extends ConsumerStatefulWidget {
  const ProviderConfigPage({super.key});

  @override
  ConsumerState<ProviderConfigPage> createState() => _ProviderConfigPageState();
}

class _ProviderConfigPageState extends ConsumerState<ProviderConfigPage>
    with SingleTickerProviderStateMixin {
  late TabController _tabController;
  late TextEditingController _apiKeyController;
  String? _lastLoadedApiKey;

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 2, vsync: this);
    _apiKeyController = TextEditingController(text: '');
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    // Watch for state changes and update text field
    final state = ref.watch(settingsNotifierProvider);
    if (state.apiKey.isNotEmpty && state.apiKey != _lastLoadedApiKey) {
      _lastLoadedApiKey = state.apiKey;
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted && _apiKeyController.text != state.apiKey) {
          _apiKeyController.text = state.apiKey;
        }
      });
    }
  }

  @override
  void dispose() {
    _tabController.dispose();
    _apiKeyController.dispose();
    // Don't destroy temp session here — it's managed by SettingsNotifier
    // and may be needed when the user returns to this page
    super.dispose();
  }

  void _saveSettings() async {
    final apiKey = _apiKeyController.text.trim();
    if (apiKey.isEmpty) {
      _showSnackBar('请输入 API Key', isError: true);
      return;
    }

    try {
      final state = ref.read(settingsNotifierProvider);
      final settingsNotifier = ref.read(settingsNotifierProvider.notifier);
      final chatNotifier = ref.read(chatNotifierProvider.notifier);

      // 保存到数据库
      await settingsNotifier.saveApiKey(state.selectedProvider, apiKey);

      // 更新 ChatViewModel 中的 API Key
      chatNotifier.updateApiKey(state.selectedProvider, apiKey);
      chatNotifier.updateProvider(state.selectedProvider);

      if (mounted) {
        _showSnackBar('设置已保存');
        Navigator.pop(context);
      }
    } catch (e) {
      if (mounted) {
        _showSnackBar('保存失败：$e', isError: true);
      }
    }
  }

  void _showSnackBar(String message, {bool isError = false}) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(message),
        backgroundColor: isError ? AppTheme.dangerColor : AppTheme.successColor,
        behavior: SnackBarBehavior.floating,
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;

    return Scaffold(
      backgroundColor: isDark ? AppTheme.darkSurface : AppTheme.lightSurface,
      appBar: AppBar(
        title: const Text('供应商配置'),
        backgroundColor: isDark ? AppTheme.darkSurface : AppTheme.lightSurface,
        bottom: TabBar(
          controller: _tabController,
          indicatorColor: AppTheme.primaryColor,
          labelColor: AppTheme.primaryColor,
          unselectedLabelColor: isDark
              ? AppTheme.darkTextSecondary
              : AppTheme.lightTextSecondary,
          tabs: const [
            Tab(text: '供应商配置'),
            Tab(text: '模型列表'),
          ],
        ),
      ),
      body: TabBarView(
        controller: _tabController,
        children: [
          _ProviderConfigTab(
            apiKeyController: _apiKeyController,
            onSave: _saveSettings,
          ),
          _ModelsTab(),
        ],
      ),
    );
  }
}

class _ProviderConfigTab extends ConsumerWidget {
  final TextEditingController apiKeyController;
  final VoidCallback onSave;

  const _ProviderConfigTab({
    required this.apiKeyController,
    required this.onSave,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(settingsNotifierProvider);

    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildSectionTitle('供应商'),
          const SizedBox(height: 8),
          _buildDropdown(
            context: context,
            value: state.selectedProvider,
            items: AppConfig.providers.map((p) {
              return DropdownMenuItem(value: p, child: Text(p.toUpperCase()));
            }).toList(),
            onChanged: (val) {
              if (val != null) {
                ref
                    .read(settingsNotifierProvider.notifier)
                    .updateProvider(val, apiKeyController.text.trim());
              }
            },
          ),
          const SizedBox(height: 24),
          _buildSectionTitle('API Key'),
          const SizedBox(height: 8),
          _buildTextField(
            context: context,
            controller: apiKeyController,
            hintText: '输入您的 API Key',
            obscureText: true,
          ),
          const SizedBox(height: 32),
          SizedBox(
            width: double.infinity,
            height: 48,
            child: ElevatedButton.icon(
              onPressed: onSave,
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
    required BuildContext context,
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
    required BuildContext context,
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
          initialValue: value,
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

class _ModelsTab extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(settingsNotifierProvider);
    final isDark = Theme.of(context).brightness == Brightness.dark;

    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: double.infinity,
            height: 48,
            child: ElevatedButton.icon(
              onPressed: state.isLoadingModels
                  ? null
                  : () async {
                      try {
                        await ref
                            .read(settingsNotifierProvider.notifier)
                            .fetchModels();
                        if (context.mounted) {
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
                        if (context.mounted) {
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
                    },
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
          SizedBox(
            width: double.infinity,
            height: 48,
            child: ElevatedButton.icon(
              onPressed: state.isLoadingBalance
                  ? null
                  : () async {
                      try {
                        await ref
                            .read(settingsNotifierProvider.notifier)
                            .fetchBalance();
                        final s = ref.read(settingsNotifierProvider);
                        if (context.mounted) {
                          ScaffoldMessenger.of(context).showSnackBar(
                            SnackBar(
                              content: Text(
                                '余额: ${s.balance['total_balance'] ?? '0'} ${s.balance['currency'] ?? ''}',
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
                        if (context.mounted) {
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
                    },
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
          const SizedBox(height: 16),
          _buildSectionTitle('模型'),
          const SizedBox(height: 8),
          if (state.models.isEmpty)
            Center(
              child: Padding(
                padding: const EdgeInsets.symmetric(vertical: 32),
                child: Text(
                  '点击"获取模型列表"加载可用模型',
                  style: TextStyle(
                    fontSize: 15,
                    color: isDark
                        ? AppTheme.darkTextSecondary
                        : AppTheme.lightTextSecondary,
                  ),
                ),
              ),
            )
          else
            Container(
              decoration: BoxDecoration(
                color: isDark ? AppTheme.darkCard : AppTheme.lightCard,
                borderRadius: BorderRadius.circular(12),
              ),
              child: Column(
                children: state.models.asMap().entries.map((entry) {
                  final index = entry.key;
                  final model = entry.value;
                  final isSelected = model == state.selectedModel;
                  return Column(
                    children: [
                      if (index > 0)
                        Padding(
                          padding: const EdgeInsets.only(left: 16),
                          child: Divider(
                            height: 1,
                            color: isDark
                                ? AppTheme.darkBorder
                                : AppTheme.lightBorderLight,
                          ),
                        ),
                      Material(
                        color: Colors.transparent,
                        child: InkWell(
                          onTap: () {
                            ref
                                .read(settingsNotifierProvider.notifier)
                                .selectModel(model);
                          },
                          child: Padding(
                            padding: const EdgeInsets.symmetric(
                              horizontal: 16,
                              vertical: 14,
                            ),
                            child: Row(
                              children: [
                                Expanded(
                                  child: Text(
                                    model,
                                    style: TextStyle(
                                      fontSize: 15,
                                      color: isDark
                                          ? AppTheme.darkText
                                          : AppTheme.lightText,
                                    ),
                                    overflow: TextOverflow.ellipsis,
                                  ),
                                ),
                                if (isSelected)
                                  const Icon(
                                    Icons.check,
                                    color: AppTheme.primaryColor,
                                    size: 20,
                                  ),
                              ],
                            ),
                          ),
                        ),
                      ),
                    ],
                  );
                }).toList(),
              ),
            ),
        ],
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
}
