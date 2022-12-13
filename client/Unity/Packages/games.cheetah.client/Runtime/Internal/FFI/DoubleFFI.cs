using System.Runtime.InteropServices;
using Games.Cheetah.Client.ServerAPI;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.Internal.FFI
{
    public static class DoubleFFI
    {
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_double_value_listener")]
        public static extern byte SetListener(ushort clientId, IDoubleServerAPI.Listener listener);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_double_value")]
        public static extern byte Set(ushort clientId, in CheetahObjectId objectId, ushort fieldId, double value);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "inc_double_value")]
        public static extern byte Increment(ushort clientId, in CheetahObjectId objectId, ushort fieldId, double increment);
    }
}