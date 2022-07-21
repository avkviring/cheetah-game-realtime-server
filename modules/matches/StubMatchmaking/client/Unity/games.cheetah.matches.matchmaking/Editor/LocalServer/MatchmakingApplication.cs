using Cheetah.Matches.Factory.Editor.LocalServer.Factory;
using Cheetah.Platform.Editor.LocalServer.Applications;
using Cheetah.Platform.Editor.LocalServer.Docker;

namespace Cheetah.Matches.Matchmaking.Editor.LocalServer
{
    public class MatchmakingApplication : PlatformApplication
    {
        public MatchmakingApplication() : base("cheetah-matches-stub-matchmaking")
        {
            ExternalGrpcServices.Add("cheetah.matches.matchmaking");
            Dependencies.Add(FactoryApplication.AppName);
        }

        public override void ConfigureDockerContainerBuilder(DockerContainerBuilder builder)
        {
            base.ConfigureDockerContainerBuilder(builder);
            builder.AddEnv(JwtKeys.PublicName, JwtKeys.PublicValue);
            builder.AddEnv("CHEETAH_MATCHES_FACTORY_INTERNAL_SERVICE_HOST", FactoryApplication.AppName);
            builder.AddEnv("CHEETAH_MATCHES_FACTORY_INTERNAL_SERVICE_PORT", InternalGrpcPort.ToString());
        }
    }
}