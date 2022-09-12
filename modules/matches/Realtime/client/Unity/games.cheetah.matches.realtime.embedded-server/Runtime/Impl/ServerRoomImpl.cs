using Cheetah.Matches.Realtime.EmbeddedServer.API;
using Cheetah.Matches.Realtime.EmbeddedServer.FFI;

namespace Cheetah.Matches.Realtime.EmbeddedServer.Impl
{
    internal class ServerRoomImpl : ServerRoom
    {
        private readonly Server.Description serverDescription;
        private readonly ulong roomId;

        internal ServerRoomImpl(Server.Description serverDescription, ulong roomId)
        {
            this.serverDescription = serverDescription;
            this.roomId = roomId;
        }

        public ServerMember CreateMember(ulong group)
        {
            var member = new Member.MemberDescription();
            Member.CreateMember(serverDescription.id, roomId, group, ref member);
            return new ServerMemberImpl(member);
        }

        public ulong GetId()
        {
            return roomId;
        }
    }
}