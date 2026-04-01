import 'package:flutter/material.dart';

class AppTheme {
  AppTheme._();

  static const primaryColor = Color(0xFF007AFF);
  static const secondaryColor = Color(0xFF5856D6);
  static const successColor = Color(0xFF34C759);
  static const dangerColor = Color(0xFFFF3B30);
  static const warningColor = Color(0xFFFFCC00);

  static const lightSurface = Color(0xFFF5F5F7);
  static const lightCard = Color(0xFFFFFFFF);
  static const lightText = Color(0xFF1D1D1F);
  static const lightTextSecondary = Color(0xFF86868B);
  static const lightBorder = Color(0xFFD2D2D7);
  static const lightBorderLight = Color(0xFFE5E5EA);
  static const lightInputBg = Color(0xFF1C1C1E);

  static const darkSurface = Color(0xFF000000);
  static const darkCard = Color(0xFF1C1C1E);
  static const darkText = Color(0xFFF5F5F7);
  static const darkTextSecondary = Color(0xFF8E8E93);
  static const darkBorder = Color(0xFF38383A);
  static const darkBorderLight = Color(0xFF2C2C2E);

  static const primaryDark = Color(0xFF0A84FF);

  static ThemeData lightTheme = ThemeData(
    useMaterial3: true,
    colorScheme: ColorScheme.light(
      primary: primaryColor,
      secondary: secondaryColor,
      surface: lightSurface,
      onSurface: lightText,
      onSurfaceVariant: lightTextSecondary,
      outline: lightBorder,
      outlineVariant: lightBorderLight,
    ),
    textTheme: const TextTheme(
      headlineLarge: TextStyle(
        fontSize: 34,
        fontWeight: FontWeight.w700,
        letterSpacing: -0.5,
        color: lightText,
      ),
      headlineMedium: TextStyle(
        fontSize: 28,
        fontWeight: FontWeight.w700,
        letterSpacing: -0.3,
        color: lightText,
      ),
      titleLarge: TextStyle(
        fontSize: 22,
        fontWeight: FontWeight.w600,
        letterSpacing: -0.3,
        color: lightText,
      ),
      bodyLarge: TextStyle(
        fontSize: 17,
        fontWeight: FontWeight.w400,
        letterSpacing: -0.4,
        color: lightText,
      ),
      bodyMedium: TextStyle(
        fontSize: 15,
        fontWeight: FontWeight.w400,
        letterSpacing: -0.2,
        color: lightText,
      ),
      bodySmall: TextStyle(
        fontSize: 13,
        fontWeight: FontWeight.w400,
        letterSpacing: -0.1,
        color: lightTextSecondary,
      ),
    ),
    appBarTheme: const AppBarTheme(
      backgroundColor: lightSurface,
      elevation: 0,
      centerTitle: true,
      titleTextStyle: TextStyle(
        fontSize: 17,
        fontWeight: FontWeight.w600,
        color: lightText,
        letterSpacing: -0.4,
      ),
    ),
  );

  static ThemeData darkTheme = ThemeData(
    useMaterial3: true,
    colorScheme: ColorScheme.dark(
      primary: primaryDark,
      secondary: secondaryColor,
      surface: darkSurface,
      onSurface: darkText,
      onSurfaceVariant: darkTextSecondary,
      outline: darkBorder,
      outlineVariant: darkBorderLight,
    ),
    textTheme: const TextTheme(
      headlineLarge: TextStyle(
        fontSize: 34,
        fontWeight: FontWeight.w700,
        letterSpacing: -0.5,
        color: darkText,
      ),
      headlineMedium: TextStyle(
        fontSize: 28,
        fontWeight: FontWeight.w700,
        letterSpacing: -0.3,
        color: darkText,
      ),
      titleLarge: TextStyle(
        fontSize: 22,
        fontWeight: FontWeight.w600,
        letterSpacing: -0.3,
        color: darkText,
      ),
      bodyLarge: TextStyle(
        fontSize: 17,
        fontWeight: FontWeight.w400,
        letterSpacing: -0.4,
        color: darkText,
      ),
      bodyMedium: TextStyle(
        fontSize: 15,
        fontWeight: FontWeight.w400,
        letterSpacing: -0.2,
        color: darkText,
      ),
      bodySmall: TextStyle(
        fontSize: 13,
        fontWeight: FontWeight.w400,
        letterSpacing: -0.1,
        color: darkTextSecondary,
      ),
    ),
    appBarTheme: const AppBarTheme(
      backgroundColor: darkSurface,
      elevation: 0,
      centerTitle: true,
      titleTextStyle: TextStyle(
        fontSize: 17,
        fontWeight: FontWeight.w600,
        color: darkText,
        letterSpacing: -0.4,
      ),
    ),
  );
}
