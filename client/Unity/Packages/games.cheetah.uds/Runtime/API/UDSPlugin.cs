using System;
using System.Collections.Generic;
using System.Linq;
using System.Net.Http;
using System.Threading.Tasks;
using Games.Cheetah.Client;
using Games.Cheetah.Client.Codec;
using Games.Cheetah.GRPC.Internal;
using Grpc.Net.Client;
using Grpc.Net.Client.Web;
using static Games.Cheetah.GRPC.Internal.Internal;

namespace Games.Cheetah.UDS.API
{
    /**
     * Плагин к cheetah серверу. Основная задача - вызов обработчика создания новой комнаты.
     * Необходимо раз в N секунд вызывать метод Update.
     */
    public class UDSPlugin
    {
        public delegate void OnRoomCreated(ulong roomId, InternalClient internalClient, NetworkClient cheetahClient);

        public delegate void OnRoomDeleted(ulong roomId);


        private readonly OnRoomCreated onRoomCreated;
        private readonly OnRoomDeleted onRoomDeleted;
        private readonly CodecRegistry codecRegistry;
        private readonly Uri webGrpcRealtimeServerInternalUri;
        private readonly string udpServerHost;
        private readonly ushort udpServerPort;
        private HashSet<ulong> processedRooms = new();
        private HashSet<ulong> tmpRooms = new();
        private readonly InternalClient internalClient;

        public UDSPlugin(
            Uri webGrpcRealtimeServerInternalUri,
            string udpServerHost,
            ushort udpServerPort,
            OnRoomCreated onRoomCreated,
            OnRoomDeleted onRoomDeleted,
            CodecRegistry codecRegistry
        )
        {
            this.webGrpcRealtimeServerInternalUri = webGrpcRealtimeServerInternalUri;
            this.udpServerHost = udpServerHost;
            this.udpServerPort = udpServerPort;
            this.onRoomCreated = onRoomCreated;
            this.onRoomDeleted = onRoomDeleted;
            this.codecRegistry = codecRegistry;
            var channel = GrpcChannel.ForAddress(
                webGrpcRealtimeServerInternalUri, new GrpcChannelOptions
                {
                    HttpHandler = new GrpcWebHandler(new HttpClientHandler()),
                }
            );

            internalClient = new InternalClient(channel);
        }


        /**
         * Необходимо вызывать для обновления состояния плагина
         */
        public async Task OnUpdate()
        {
            var rooms = await internalClient.GetRoomsAsync(new EmptyRequest());
            var roomsOnServer = rooms.Rooms.ToHashSet();

            foreach (var room in roomsOnServer)
            {
                if (processedRooms.Contains(room)) continue;

                processedRooms.Add(room);
                await CreateRoomPlugin(room);
            }

            var roomsToRemove = new HashSet<ulong>();
            foreach (var room in processedRooms)
            {
                if (roomsOnServer.Contains(room)) continue;

                roomsToRemove.Add(room);
            }

            foreach (var room in roomsToRemove)
            {
	            processedRooms.Remove(room);
	            onRoomDeleted(room);
            }
        }

        private async Task CreateRoomPlugin(ulong roomId)
        {
            var member = await internalClient.CreateSuperMemberAsync(new CreateSuperMemberRequest
            {
                RoomId = roomId
            });

            var cheetahClient = new NetworkClient(
                0,
                udpServerHost,
                udpServerPort,
                member.UserId,
                roomId,
                member.PrivateKey.ToByteArray(),
                codecRegistry);

            onRoomCreated(roomId, internalClient, cheetahClient);
        }
    }
}
