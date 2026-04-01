class AppConfig {
  AppConfig._();

  static const String appName = 'Jasmine AI';
  static const String appVersion = '1.0.0';

  static const List<String> providers = [
    'openrouter',
    'deepseek',
    'siliconflow',
  ];

  static const int defaultMaxTokens = 4096;
}
