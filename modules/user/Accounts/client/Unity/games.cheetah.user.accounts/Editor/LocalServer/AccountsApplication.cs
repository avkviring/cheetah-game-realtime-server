using Cheetah.Platform.Editor.LocalServer.Applications;
using Cheetah.Platform.Editor.LocalServer.Docker;

namespace Cheetah.User.Accounts.Editor.LocalServer
{
    public class AccountsApplication : PlatformApplication
    {
        public const string AppName = "user-accounts";

        private string RedisName;

        public AccountsApplication(string redisName) : base(AppName)
        {
            YDBEnabled = true;
            RedisName = redisName;
            ExternalGrpcServices.Add("cheetah.accounts");
            Dependencies.Add(redisName);
        }


        public override void ConfigureDockerContainerBuilder(DockerContainerBuilder builder)
        {
            base.ConfigureDockerContainerBuilder(builder);
            // var appId = GooglePlayGamesSettings.GetOrCreateSettings().AppId;
            // if (appId.Trim().Length > 0)
            // {
            //     builder.AddEnv("AUTH_GOOGLE_CLIENT_ID", appId);
            // }
            builder.AddEnv(JwtKeys.PublicName, JwtKeys.PublicValue);
            builder.AddEnv(JwtKeys.PrivateName, JwtKeys.PrivateValue);
            builder.AddEnv("REDIS_HOST", RedisName);
            builder.AddEnv("REDIS_PORT", "6379");
        }
    }
}