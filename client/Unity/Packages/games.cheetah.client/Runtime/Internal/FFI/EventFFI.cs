using System.Runtime.InteropServices;
using Games.Cheetah.Client.ServerAPI;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.Internal.FFI
{
    public static class EventFFI
    {
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_event_listener")]
        public static extern byte SetListener(ushort clientId, IEventServerAPI.Listener listener);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "send_event")]
        public static extern byte Send(ushort clientId, in CheetahObjectId objectId, ushort fieldId, ref CheetahBuffer data);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "send_target_event")]
        public static extern byte Send(ushort clientId, ushort targetUser, in CheetahObjectId objectId, ushort fieldId, ref CheetahBuffer data);
    }
}