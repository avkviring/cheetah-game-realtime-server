using System.Runtime.InteropServices;
using Games.Cheetah.Client.Types.Command;
using Games.Cheetah.Client.Types.Field;
using Games.Cheetah.Client.Types.Network;
using Games.Cheetah.Client.Types.Object;

namespace Games.Cheetah.Client.Internal.FFI
{
    public static class FFIMethods
    {
#if UNITY_IOS
        public const string Library = "__Internal";
#else
        public const string Library = "cheetah_client";
#endif
        public const ushort MaxSizeStruct = 255;
        public const ushort MaxFields = 255;

        [DllImport(Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "create_client")]
        public static extern byte CreateClient(
            [MarshalAs(UnmanagedType.LPStr)] string serverAddress,
            ushort memberId,
            ulong roomId,
            ref NetworkBuffer userPrivateKey,
            ulong startFrameId,
            out ushort clientId
        );


        [DllImport(Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "get_connection_status")]
        public static extern byte GetConnectionStatus(ushort clientId, out ConnectionStatus status);

        [DllImport(Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "get_statistics")]
        public static extern byte GetStatistics(ushort clientId, out Statistics clientStatistics);


        [DllImport(Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "receive")]
        public static extern unsafe byte Receive(ushort clientId, S2CCommand* commands, ref byte count);


        [DllImport(Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "destroy_client")]
        public static extern byte DestroyClient(ushort clientId);


        [DllImport(dllName: Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "attach_to_room")]
        public static extern byte AttachToRoom(ushort clientId);

        [DllImport(Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "detach_from_room")]
        public static extern byte DetachFromRoom(ushort clientId);

        [DllImport(Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_channel")]
        public static extern byte SetChannelType(ushort clientId, ChannelType channelType, byte group);

        [DllImport(Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_rtt_emulation")]
        public static extern byte SetRttEmulation(ushort clientId, ulong rttInMs, double rttDispersion);

        [DllImport(Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_drop_emulation")]
        public static extern byte SetDropEmulation(ushort clientId, double dropProbability, ulong dropTimeInMs);

        [DllImport(Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "reset_emulation")]
        public static extern byte ResetEmulation(ushort clientId);

        [DllImport(Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "get_last_error_msg")]
        public static extern void GetLastErrorMsg(ref NetworkBuffer buffer);

        [DllImport(Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "get_server_time")]
        public static extern byte GetServerTime(ushort clientId, out ulong time);

        [DllImport(Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_double_value")]
        public static extern byte Set(ushort clientId, in NetworkObjectId objectId, ushort fieldId, double value);

        [DllImport(Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "inc_double_value")]
        public static extern byte Increment(ushort clientId, in NetworkObjectId objectId, ushort fieldId, double increment);

        [DllImport(Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "send_event")]
        public static extern byte Send(ushort clientId, in NetworkObjectId objectId, ushort fieldId, ref NetworkBuffer data);

        [DllImport(Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "send_target_event")]
        public static extern byte Send(ushort clientId, ushort targetUser, in NetworkObjectId objectId, ushort fieldId, ref NetworkBuffer data);

        [DllImport(Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "delete_field")]
        public static extern byte DeleteField(ushort clientId, in NetworkObjectId objectId, ushort fieldId, FieldType value);

        [DllImport(Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_long_value")]
        public static extern byte Set(ushort clientId, in NetworkObjectId objectId, ushort fieldId, long value);

        [DllImport(Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "inc_long_value")]
        public static extern byte Increment(ushort clientId, in NetworkObjectId objectId, ushort fieldId, long increment);

        [DllImport(Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "create_object")]
        public static extern byte CreateObject(ushort clientId, ushort template, ulong accessGroup, ref NetworkObjectId objectId);

        [DllImport(Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "created_object")]
        public static extern byte CreatedObject(ushort clientId, in NetworkObjectId objectId, bool roomOwner, ref NetworkBuffer singletonKey);

        [DllImport(Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "delete_object")]
        public static extern byte DeleteObject(ushort clientId, in NetworkObjectId objectId);

        [DllImport(Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_structure")]
        public static extern byte Set(ushort clientId, in NetworkObjectId objectId, ushort fieldId, ref NetworkBuffer data);
    }
}