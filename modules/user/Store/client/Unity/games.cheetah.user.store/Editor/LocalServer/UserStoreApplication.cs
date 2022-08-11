using Cheetah.Platform.Editor.LocalServer.Applications;
using Cheetah.Platform.Editor.LocalServer.Docker;

namespace Cheetah.User.Store.Editor.LocalServer
{
    public class UserStoreApplication : PlatformApplication
    {
        public const string AppName = "user-store";

        public UserStoreApplication() : base(AppName)
        {
            ExternalGrpcServices.Add("cheetah.user.store");
            EnablePostgreSQL("user_store");
        }

        public override void ConfigureDockerContainerBuilder(DockerContainerBuilder builder)
        {
            base.ConfigureDockerContainerBuilder(builder);
            base.ConfigurePostgresEnv(builder);
            builder.AddEnv(JwtKeys.PublicName, JwtKeys.PublicValue);
        }
    }
}