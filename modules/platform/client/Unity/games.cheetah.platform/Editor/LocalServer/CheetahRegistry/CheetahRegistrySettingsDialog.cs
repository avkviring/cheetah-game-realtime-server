using UnityEditor;
using UnityEngine.UIElements;

namespace Cheetah.Platform.Editor.LocalServer.CheetahRegistry
{
    public class CheetahRegistrySettingsDialog : VisualElement
    {
        public delegate void OnSuccess();

        private readonly Button Check;
        private readonly HelpBox CheckResult;
        private readonly TextField Login;
        private readonly TextField Password;
        private VisualElement RootElement;

        public CheetahRegistrySettingsDialog()
        {
            var uiAsset =
                AssetDatabase.LoadAssetAtPath<VisualTreeAsset>("Packages/games.cheetah.platform/Editor/LocalServer/CheetahRegistry/UI.uxml");
            uiAsset.CloneTree(this);

            Login = this.Q<TextField>("login");
            var cheetahRegistrySettings = CheetahRegistrySettingsFromPrefs.Instance;
            Login.value = cheetahRegistrySettings.Login;
            Login.RegisterValueChangedCallback(e =>
            {
                cheetahRegistrySettings.Login = e.newValue;
                ResetCheckMessage();
            });

            Password = this.Q<TextField>("password");
            Password.value = cheetahRegistrySettings.Password;
            Password.RegisterValueChangedCallback(e =>
            {
                cheetahRegistrySettings.Password = e.newValue;
                ResetCheckMessage();
            });


            Check = this.Q<Button>("check");
            Check.RegisterCallback<ClickEvent>(async e =>
            {
                if (await new CheetachDockerRegistry(cheetahRegistrySettings).CheckAuth())
                    CheckPassed();
                else
                    CheckFail();
            });
            CheckResult = this.Q<HelpBox>("check-result");
            ResetCheckMessage();
        }

        public event OnSuccess OnSuccessEvent;


        private void CheckPassed()
        {
            OnSuccessEvent?.Invoke();
            CheckResult.messageType = HelpBoxMessageType.Info;
            CheckResult.text = "Stored. Username and password accepted.";
            CheckResult.style.display = new StyleEnum<DisplayStyle>(DisplayStyle.Flex);
        }

        private void CheckFail()
        {
            CheckResult.messageType = HelpBoxMessageType.Error;
            CheckResult.text = "Username/password invalid or server not available.";
            CheckResult.style.display = new StyleEnum<DisplayStyle>(DisplayStyle.Flex);
        }

        private void ResetCheckMessage()
        {
            CheckResult.style.display = new StyleEnum<DisplayStyle>(DisplayStyle.None);
        }
    }
}