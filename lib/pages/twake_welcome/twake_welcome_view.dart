import 'package:fluffychat/config/app_config.dart';
import 'package:fluffychat/pages/twake_welcome/twake_welcome.dart';
import 'package:fluffychat/pages/twake_welcome/welcome_screen.dart';
import 'package:fluffychat/widgets/twake_components/twake_icon_button.dart';
import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/l10n.dart';
import 'package:go_router/go_router.dart';

class TwakeWelcomeView extends StatelessWidget {
  final TwakeWelcomeController controller;

  const TwakeWelcomeView({super.key, required this.controller});

  @override
  Widget build(BuildContext context) {
    return WelcomeScreen(
      appBar: controller.widget.arg?.isAddAnotherAccount == true
          ? AppBar(
              backgroundColor: Colors.transparent,
              leading: TwakeIconButton(
                icon: Icons.chevron_left_outlined,
                onTap: () => context.pop(),
                tooltip: L10n.of(context)!.back,
              ),
              elevation: 0,
            )
          : null,
      focusColor: Colors.transparent,
      hoverColor: Colors.transparent,
      highlightColor: Colors.transparent,
      overlayColor: WidgetStateProperty.all(Colors.transparent),
      signInTitle: AppConfig.isSaasPlatForm ? L10n.of(context)!.signIn : null,
      createTwakeIdTitle:
          AppConfig.isSaasPlatForm ? L10n.of(context)!.signUp : null,
      // useCompanyServerTitle: L10n.of(context)!.useYourCompanyServer,
      description: L10n.of(context)!.descriptionTwakeId,
      // onUseCompanyServerOnTap: controller.goToHomeserverPicker,
      onSignInOnTap:
          AppConfig.isSaasPlatForm ? controller.onClickAuthorize : null,
      privacyPolicy: L10n.of(context)!.privacyPolicy,
      descriptionPrivacyPolicy: L10n.of(context)!.byContinuingYourAgreeingToOur,
      onPrivacyPolicyOnTap: controller.onClickPrivacyPolicy,
      loading: controller.loading,
      onSignUpOnTap:
          AppConfig.isSaasPlatForm ? controller.onClickAuthorize : null,
      // logo: SvgPicture.asset(
      //   ImagePaths.logoTwakeWelcome,
      //   width: TwakeWelcomeViewStyle.logoWidth,
      //   height: TwakeWelcomeViewStyle.logoHeight,
      // ),
    );
  }
}
