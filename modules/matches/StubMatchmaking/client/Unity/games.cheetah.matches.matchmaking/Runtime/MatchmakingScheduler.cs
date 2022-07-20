using System.Threading.Tasks;
using Cheetah.Accounts;
using Cheetah.Matches.Matchmaking.GRPC;

namespace Cheetah.Matches.Matchmaking
{
    public class MatchmakingScheduler
    {
        private readonly User user;

        public MatchmakingScheduler(User user)
        {
            this.user = user;
        }

        public async Task<TicketResponse> Schedule(string roomTemplate, ulong userGroups)
        {
            return await user.ClusterConnector.DoRequest(async channel =>
            {
                var client = new GRPC.Matchmaking.MatchmakingClient(channel);
                var ticketRequest = new TicketRequest { UserGroups = userGroups, MatchTemplate = roomTemplate };
                var response = await client.MatchmakingAsync(ticketRequest, user.CreateAuthMetadata());
                return response;
            });
        }
    }
}