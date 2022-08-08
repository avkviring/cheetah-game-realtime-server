using Cheetah.Platform.Editor.LocalServer.Applications;
using Cheetah.Platform.Editor.LocalServer.Docker;

namespace Cheetah.User.Accounts.Editor.LocalServer
{
    public class UserStoreApplication : PlatformApplication
    {
        public const string AppName = "user-store";

        public UserStoreApplication() : base(AppName)
        {
            YDBEnabled = true;
            ExternalGrpcServices.Add("cheetah.user.store");
        }

        public override void ConfigureDockerContainerBuilder(DockerContainerBuilder builder)
        {
            base.ConfigureDockerContainerBuilder(builder);
            builder.AddEnv(JwtKeys.PublicName, JwtKeys.PublicValue);
        }
    }
}
