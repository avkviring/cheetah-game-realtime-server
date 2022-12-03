using System;
using System.Net.Http;
using Cheetah.Matches.Realtime.Codec;
using Cheetah.Matches.Realtime.GRPC.Internal;
using Grpc.Net.Client;
using Grpc.Net.Client.Web;
using static Cheetah.Matches.Realtime.GRPC.Internal.Realtime;

namespace Cheetah.Matches.Realtime.UDS.API
{
    public class Plugin
    {
        public delegate void OnRoomCreated(ulong roomId, RealtimeClient grpcClient, CheetahClient realtimeClient);

        public delegate void OnRoomDeleted(ulong roomId);


        private readonly OnRoomCreated onRoomCreated;
        private readonly OnRoomDeleted onRoomDeleted;
        private readonly CodecRegistry codecRegistry;
        private ushort serverPluginId;
        private readonly string grpcRealtimeServerInternalAddress;
        private readonly string udpRealtimeServerAddress;


        public Plugin(
            string grpcRealtimeServerInternalAddress,
            string udpRealtimeServerAddress,
            OnRoomCreated onRoomCreated,
            OnRoomDeleted onRoomDeleted,
            CodecRegistry codecRegistry
        )
        {
            this.grpcRealtimeServerInternalAddress = grpcRealtimeServerInternalAddress;
            this.udpRealtimeServerAddress = udpRealtimeServerAddress;
            this.onRoomCreated = onRoomCreated;
            this.onRoomDeleted = onRoomDeleted;
            this.codecRegistry = codecRegistry;
            var result = FFI.Plugin.CreatePlugin(grpcRealtimeServerInternalAddress, out serverPluginId);
            if (result != FFI.Plugin.ResultCode.OK)
            {
                ThrowLastError();
            }
        }


        public void OnUpdate()
        {
            var result = UDS.FFI.Plugin.PopRoomEvent(serverPluginId, out var roomEvent);
            switch (result)
            {
                case UDS.FFI.Plugin.ResultCode.OK:
                    switch (roomEvent.eventType)
                    {
                        case UDS.FFI.Plugin.RoomEventType.Created:
                            CreateRoomPlugin(roomEvent.roomId);
                            break;
                        case UDS.FFI.Plugin.RoomEventType.Deleted:
                            onRoomDeleted.Invoke(roomEvent.roomId);
                            break;
                        default:
                            throw new ArgumentOutOfRangeException();
                    }

                    break;
                case UDS.FFI.Plugin.ResultCode.Empty:
                    break;
                case UDS.FFI.Plugin.ResultCode.Error:
                    ThrowLastError();
                    break;
                default:
                    throw new ArgumentOutOfRangeException();
            }
        }

        private async void CreateRoomPlugin(ulong roomId)
        {
            var channel = GrpcChannel.ForAddress(
                grpcRealtimeServerInternalAddress, new GrpcChannelOptions
                {
                    HttpHandler = new GrpcWebHandler(new HttpClientHandler()),
                }
            );

            var grpcClient = new RealtimeClient(channel);
            var member = await grpcClient.CreateSuperMemberAsync(new CreateSuperMemberRequest
            {
                RoomId = roomId
            });

            var realtimeClient = new CheetahClient(
                udpRealtimeServerAddress,
                member.UserId,
                roomId,
                member.PrivateKey.ToByteArray(),
                codecRegistry);

            onRoomCreated(roomId, grpcClient, realtimeClient);
        }

        private static unsafe void ThrowLastError()
        {
            FFI.Plugin.GetLastErrorMsg(out var nativeString);
            throw new Exception(new string(nativeString.values, 0, nativeString.size));
        }
    }
}