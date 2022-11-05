using System.Threading.Tasks;
using Cheetah.Matches.Matchmaking;
using Cheetah.Matches.Matchmaking.GRPC;
using Cheetah.Platform;

namespace Shared
{
    public class PlayerHelper
    {
        /// <summary>
        /// Создаем нового игрока и закидываем его в битву
        /// </summary>
        /// <param name="clusterConnector"></param>
        /// <returns></returns>
        public static async Task<TicketResponse> CreateNewPlayerAndMatchToBattle(ClusterConnector clusterConnector, string prefix)
        {
            // сообщаем mm о желании попасть в битву
            var scheduler = new MatchmakingScheduler(clusterConnector);
            var ticket = await scheduler.Schedule("gubaha", PlayerGroup);
            return ticket;
        }

        public const ulong PlayerGroup = 8;
    }
}