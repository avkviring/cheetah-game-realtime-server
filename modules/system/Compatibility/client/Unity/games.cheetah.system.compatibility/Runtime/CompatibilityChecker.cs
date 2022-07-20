using System.Threading.Tasks;
using Cheetah.Platform;
using Cheetah.System.Compatibility.GRPC;

namespace Cheetah.System.Compatibility
{
    /**
     * Проверка статуса совместимости клиента и сервера
     */
    public class CompatibilityChecker
    {
        private readonly ClusterConnector connector;

        public CompatibilityChecker(ClusterConnector connector)
        {
            this.connector = connector;
        }


        public async Task<CheckCompatibilityResponse.Types.Status> Check(string clientVersion)
        {
            return await connector.DoRequest(async channel =>
            {
                var client = new GRPC.CompatibilityChecker.CompatibilityCheckerClient(channel);
                var checkCompatibilityRequest = new CheckCompatibilityRequest
                {
                    Version = clientVersion
                };
                var result = await client.CheckCompatibilityAsync(checkCompatibilityRequest);
                return result.Status;
            });
        }
    }
}