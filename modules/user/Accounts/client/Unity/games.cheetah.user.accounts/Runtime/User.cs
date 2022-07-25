using Cheetah.Platform;
using Grpc.Core;

namespace Cheetah.User.Accounts
{
    public class User
    {
        public readonly ClusterConnector ClusterConnector;
        public string SessionToken { get; }
        public string RefreshToken { get; }

        public User(ClusterConnector clusterConnector, string sessionToken, string refreshToken)
        {
            ClusterConnector = clusterConnector;
            SessionToken = sessionToken;
            RefreshToken = refreshToken;
        }


        public override string ToString()
        {
            return "Player [" + SessionToken + "@" + RefreshToken + "]";
        }

        public Metadata CreateAuthMetadata()
        {
            return new Metadata { { "authorization", "Bearer " + SessionToken } };
        }
    }
}