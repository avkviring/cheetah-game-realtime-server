using System.Runtime.InteropServices;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.Internal.FFI
{
    internal static class ClientFFI
    {
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "create_client")]
        public static extern byte CreateClient(
            [MarshalAs(UnmanagedType.LPStr)] string serverAddress,
            ushort memberId,
            ulong roomId,
            ref CheetahBuffer userPrivateKey,
            ulong startFrameId,
            out ushort clientId
        );


        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "get_connection_status")]
        public static extern byte GetConnectionStatus(ushort clientId, out CheetahClientConnectionStatus status);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "get_statistics")]
        public static extern byte GetStatistics(ushort clientId, out CheetahClientStatistics clientStatistics);


        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "receive")]
        public static extern byte Receive(ushort clientId);


        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "destroy_client")]
        public static extern byte DestroyClient(ushort clientId);


        [DllImport(dllName: Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "attach_to_room")]
        public static extern byte AttachToRoom(ushort clientId);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "detach_from_room")]
        public static extern byte DetachFromRoom(ushort clientId);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_channel")]
        public static extern byte SetChannelType(ushort clientId, ChannelType channelType, byte group);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_rtt_emulation")]
        public static extern byte SetRttEmulation(ushort clientId, ulong rttInMs, double rttDispersion);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_drop_emulation")]
        public static extern byte SetDropEmulation(ushort clientId, double dropProbability, ulong dropTimeInMs);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "reset_emulation")]
        public static extern byte ResetEmulation(ushort clientId);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "get_last_error_msg")]
        public static extern void GetLastErrorMsg(ref CheetahBuffer buffer);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "get_server_time")]
        public static extern byte GetServerTime(ushort clientId, out ulong time);
    }
}