import 'package:flutter/material.dart';
import 'src/rust/bindings.dart';
import 'dart:ffi';

void main() {
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Claude Core Test',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.deepPurple),
        useMaterial3: true,
      ),
      home: const RustTestPage(),
    );
  }
}

class RustTestPage extends StatefulWidget {
  const RustTestPage({super.key});

  @override
  State<RustTestPage> createState() => _RustTestPageState();
}

class _RustTestPageState extends State<RustTestPage> {
  ClaudeCore? _claudeCore;
  Pointer<Void>? _session;
  final TextEditingController _messageController = TextEditingController();
  String _response = '';
  bool _isLoading = false;

  @override
  void initState() {
    super.initState();
    _initRust();
  }

  void _initRust() {
    try {
      _claudeCore = ClaudeCore();
      
      // Initialize session with config
      final config = {
        'provider': 'openrouter',
        'model': 'anthropic/claude-3.5-sonnet',
        'max_tokens': 2048,
      };
      
      _session = _claudeCore!.createSession(config);
      
      setState(() {
        _response = 'Rust library initialized successfully!';
      });
    } catch (e) {
      setState(() {
        _response = 'Error initializing Rust: $e';
      });
    }
  }

  void _sendMessage() async {
    if (_claudeCore == null || _session == null) {
      setState(() {
        _response = 'Rust library not initialized';
      });
      return;
    }

    final message = _messageController.text;
    if (message.isEmpty) return;

    setState(() {
      _isLoading = true;
      _response = 'Sending message...';
    });

    try {
      final response = _claudeCore!.sendMessage(_session!, message);
      setState(() {
        _response = 'Response: $response';
        _isLoading = false;
      });
    } catch (e) {
      setState(() {
        _response = 'Error: $e';
        _isLoading = false;
      });
    }
  }

  void _testProvider() {
    if (_claudeCore == null || _session == null) return;

    try {
      final success = _claudeCore!.setProvider(
        _session!,
        'deepseek',
        'test-api-key',
      );
      
      setState(() {
        _response = 'Provider set: $success';
      });
    } catch (e) {
      setState(() {
        _response = 'Error setting provider: $e';
      });
    }
  }

  @override
  void dispose() {
    if (_session != null && _claudeCore != null) {
      _claudeCore!.destroySession(_session!);
    }
    _messageController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Claude Core Rust Test'),
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
      ),
      body: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          children: [
            TextField(
              controller: _messageController,
              decoration: const InputDecoration(
                labelText: 'Message',
                border: OutlineInputBorder(),
              ),
            ),
            const SizedBox(height: 16),
            Row(
              children: [
                ElevatedButton(
                  onPressed: _isLoading ? null : _sendMessage,
                  child: _isLoading ? const CircularProgressIndicator() : const Text('Send'),
                ),
                const SizedBox(width: 8),
                ElevatedButton(
                  onPressed: _testProvider,
                  child: const Text('Test Provider'),
                ),
              ],
            ),
            const SizedBox(height: 16),
            Expanded(
              child: Container(
                width: double.infinity,
                padding: const EdgeInsets.all(12),
                decoration: BoxDecoration(
                  color: Colors.grey[200],
                  borderRadius: BorderRadius.circular(8),
                ),
                child: SingleChildScrollView(
                  child: Text(
                    _response,
                    style: const TextStyle(fontFamily: 'monospace'),
                  ),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
