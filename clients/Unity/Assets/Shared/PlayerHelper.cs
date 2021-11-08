using System.Threading.Tasks;
using Cheetah.Auth.Cookie;
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
        public static async Task<TicketResponse> CreateNewPlayerAndMatchToBattle(ClusterConnector clusterConnector)
        {
            // создаем нового пользователя
            var cookieAuthenticator = new CookieAuthenticator(clusterConnector, "user1");
            cookieAuthenticator.RemoveLocalCookie();
            var loginOrRegister = await cookieAuthenticator.LoginOrRegister();

            // сообщаем mm о желании попасть в битву
            var player = loginOrRegister.Player;
            var scheduler = new MatchmakingScheduler(player);
            var ticket = await scheduler.Schedule("gubaha", UserGroup);
            return ticket;
        }

        public const ulong UserGroup = 7;
    }
}