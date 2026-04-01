import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../services/services.dart';
import '../repositories/repositories.dart';

final claudeCoreProvider = Provider<ClaudeCore>((ref) {
  return ClaudeCore();
});

final sessionRepositoryProvider = Provider<SessionRepository>((ref) {
  final claudeCore = ref.watch(claudeCoreProvider);
  return SessionRepository(claudeCore);
});
