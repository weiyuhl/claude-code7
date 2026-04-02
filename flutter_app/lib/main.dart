import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path_provider/path_provider.dart';
import 'src/core/app_theme.dart';
import 'src/core/app_config.dart';
import 'src/views/views.dart';
import 'src/services/services.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  // 初始化数据库
  try {
    final dir = await getApplicationDocumentsDirectory();
    final dbPath = '${dir.path}/claude_core.db';
    // Use the shared ClaudeCore instance from the provider, don't create a new one
    final claudeCore = ClaudeCore();
    final result = claudeCore.initDatabase(dbPath);
    if (result == 0) {
      debugPrint('数据库初始化成功：$dbPath');
    } else {
      debugPrint('数据库初始化失败：$dbPath');
    }
  } catch (e) {
    debugPrint('数据库初始化异常：$e');
  }

  runApp(const ProviderScope(child: MyApp()));
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      debugShowCheckedModeBanner: false,
      title: AppConfig.appName,
      theme: AppTheme.lightTheme,
      darkTheme: AppTheme.darkTheme,
      themeMode: ThemeMode.system,
      home: const ChatPage(),
    );
  }
}
