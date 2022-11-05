using System.Threading.Tasks;
using Cheetah.Matches.Matchmaking.GRPC;
using Cheetah.Platform;

namespace Cheetah.Matches.Matchmaking
{
    public class MatchmakingScheduler
    {
        private readonly ClusterConnector clusterConnector;

        public MatchmakingScheduler(ClusterConnector clusterConnector)
        {
            this.clusterConnector = clusterConnector;
        }

        public async Task<TicketResponse> Schedule(string roomTemplate, ulong userGroups)
        {
            return await clusterConnector.DoRequest(async channel =>
            {
                var client = new GRPC.Matchmaking.MatchmakingClient(channel);
                var ticketRequest = new TicketRequest { UserGroups = userGroups, MatchTemplate = roomTemplate };
                var response = await client.MatchmakingAsync(ticketRequest);
                return response;
            });
        }
    }
}