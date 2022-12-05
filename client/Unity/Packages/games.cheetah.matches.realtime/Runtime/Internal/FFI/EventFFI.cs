using System.Runtime.InteropServices;
using Cheetah.Matches.Realtime.Types;

namespace Cheetah.Matches.Realtime.Internal.FFI
{
    internal static class EventFFI
    {
        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void Listener(ushort commandCreator, in CheetahObjectId objectId, ushort fieldId, ref CheetahBuffer data);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_event_listener")]
        public static extern byte SetListener(ushort clientId, Listener listener);
        
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "send_event")]
        public static extern byte Send(ushort clientId, in CheetahObjectId objectId, ushort fieldId, ref CheetahBuffer data);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "send_target_event")]
        public static extern byte Send(ushort clientId, ushort targetUser, in CheetahObjectId objectId, ushort fieldId, ref CheetahBuffer data);
    }
}