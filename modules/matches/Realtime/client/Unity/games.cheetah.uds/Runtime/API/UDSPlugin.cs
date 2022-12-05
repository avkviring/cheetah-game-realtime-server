using System;
using System.Net.Http;
using System.Threading.Tasks;
using Cheetah.Matches.Realtime;
using Cheetah.Matches.Realtime.Codec;
using Cheetah.Matches.Realtime.GRPC.Internal;
using Games.Cheetah.UDS.FFI;
using Grpc.Net.Client;
using Grpc.Net.Client.Web;
using static Cheetah.Matches.Realtime.GRPC.Internal.Realtime;

namespace Games.Cheetah.UDS.API
{
    /**
     * Плагин к cheetah серверу. Основная задача - вызов обработчика создания новой комнаты.
     * Необходимо раз в N секунд вызывать метод Update.
     */
    public class UDSPlugin
    {
        public delegate void OnRoomCreated(ulong roomId, RealtimeClient grpcClient, CheetahClient cheetahClient);

        public delegate void OnRoomDeleted(ulong roomId);


        private readonly OnRoomCreated onRoomCreated;
        private readonly OnRoomDeleted onRoomDeleted;
        private readonly CodecRegistry codecRegistry;
        private readonly ushort serverPluginId;
        private readonly Uri grpcRealtimeServerInternalUri;
        private readonly Uri webGrpcRealtimeServerInternalUri;
        private readonly Uri udpRealtimeServerUri;

        public UDSPlugin(
            Uri grpcRealtimeServerInternalUri,
            Uri webGrpcRealtimeServerInternalUri,
            Uri udpRealtimeServerUri,
            OnRoomCreated onRoomCreated,
            OnRoomDeleted onRoomDeleted,
            CodecRegistry codecRegistry
        )
        {
            this.grpcRealtimeServerInternalUri = grpcRealtimeServerInternalUri;
            this.webGrpcRealtimeServerInternalUri = webGrpcRealtimeServerInternalUri;
            this.udpRealtimeServerUri = udpRealtimeServerUri;
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
                            throw new ArgumentOutOfRangeException();
                    }

                    break;
                case Plugin.ResultCode.Empty:
                    break;
                case Plugin.ResultCode.Error:
                    ThrowLastError();
                    break;
                default:
                    throw new ArgumentOutOfRangeException();
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

            var grpcClient = new RealtimeClient(channel);
            var member = await grpcClient.CreateSuperMemberAsync(new CreateSuperMemberRequest
            {
                RoomId = roomId
            });
            
            
            var cheetahClient = new CheetahClient(
                udpRealtimeServerUri,
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