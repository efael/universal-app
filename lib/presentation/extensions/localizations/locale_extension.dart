import 'package:flutter_gen/gen_l10n/l10n.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_localized_locales/flutter_localized_locales.dart';

extension LocaleExtension on Locale {
  String getLanguageNameByCurrentLocale(BuildContext context) {
    switch (languageCode) {
      case 'uz':
        return LocaleNames.of(context)!.nameOf('uz') ??
            L10n.of(context)!.languageVietnamese;
      case 'ru':
        return LocaleNames.of(context)!.nameOf('ru') ??
            L10n.of(context)!.languageRussian;
      case 'en':
        return LocaleNames.of(context)!.nameOf('en') ??
            L10n.of(context)!.languageEnglish;
      default:
        return '';
    }
  }

  String getSourceLanguageName() {
    switch (languageCode) {
      case 'en':
        return 'English';
      case 'uz':
        return 'O‘zbekcha';
      case 'ru':
        return 'Русский';
      default:
        return '';
    }
  }
}
