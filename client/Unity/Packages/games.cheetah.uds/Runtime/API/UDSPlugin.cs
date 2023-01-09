using System;
using System.Net.Http;
using System.Threading.Tasks;
using Games.Cheetah.Client;
using Games.Cheetah.Client.Codec;
using Games.Cheetah.GRPC.Internal;
using Games.Cheetah.UDS.FFI;
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
        private readonly ushort serverPluginId;
        private readonly Uri grpcRealtimeServerInternalUri;
        private readonly Uri webGrpcRealtimeServerInternalUri;
        private readonly string udpServerHost;
        private readonly ushort udpServerPort;

        public UDSPlugin(
            Uri grpcRealtimeServerInternalUri,
            Uri webGrpcRealtimeServerInternalUri,
            string udpServerHost,
            ushort udpServerPort,
            OnRoomCreated onRoomCreated,
            OnRoomDeleted onRoomDeleted,
            CodecRegistry codecRegistry
        )
        {
            this.grpcRealtimeServerInternalUri = grpcRealtimeServerInternalUri;
            this.webGrpcRealtimeServerInternalUri = webGrpcRealtimeServerInternalUri;
            this.udpServerHost = udpServerHost;
            this.udpServerPort = udpServerPort;
            this.onRoomCreated = onRoomCreated;
            this.onRoomDeleted = onRoomDeleted;
            this.codecRegistry = codecRegistry;
            var result = Plugin.CreatePlugin(grpcRealtimeServerInternalUri.AbsoluteUri, out serverPluginId);
            if (result != Plugin.ResultCode.OK)
            {
                ThrowLastError();
            }
        }


        /**
         * Необходимо вызывать для обновления состояния плагина
         */
        public async Task OnUpdate()
        {
            var result = Plugin.PopRoomEvent(serverPluginId, out var roomEvent);
            switch (result)
            {
                case Plugin.ResultCode.OK:
                    switch (roomEvent.eventType)
                    {
                        case Plugin.RoomEventType.Created:
                            await CreateRoomPlugin(roomEvent.roomId);
                            break;
                        case Plugin.RoomEventType.Deleted:
                            onRoomDeleted.Invoke(roomEvent.roomId);
                            break;
                        default:
                            throw new ArgumentOutOfRangeException("EventType " + roomEvent.eventType);
                    }

                    break;
                case Plugin.ResultCode.Empty:
                    break;
                case Plugin.ResultCode.Error:
                    ThrowLastError();
                    break;
                default:
                    throw new ArgumentOutOfRangeException("ResultCode "+result);
            }
        }

        private async Task CreateRoomPlugin(ulong roomId)
        {
            var channel = GrpcChannel.ForAddress(
                webGrpcRealtimeServerInternalUri, new GrpcChannelOptions
                {
                    HttpHandler = new GrpcWebHandler(new HttpClientHandler()),
                }
            );

            var grpcClient = new InternalClient(channel);
            var member = await grpcClient.CreateSuperMemberAsync(new CreateSuperMemberRequest
            {
                RoomId = roomId
            });


            var cheetahClient = new NetworkClient(
                udpServerHost,
                udpServerPort,
                member.UserId,
                roomId,
                member.PrivateKey.ToByteArray(),
                codecRegistry);

            onRoomCreated(roomId, grpcClient, cheetahClient);
        }

        private static unsafe void ThrowLastError()
        {
            Plugin.GetLastErrorMsg(out var nativeString);
            throw new Exception(new string(nativeString.values, 0, nativeString.size));
        }
    }
}