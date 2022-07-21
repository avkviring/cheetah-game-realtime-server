using System.Runtime.InteropServices;
using Cheetah.Matches.Relay.Types;

namespace Cheetah.Matches.Relay.Internal.FFI
{
    internal static class DoubleFFI
    {
        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void Listener(ushort commandCreator, ref CheetahObjectId objectId, ushort fieldId, double value);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_double_value_listener")]
        public static extern byte SetListener(ushort clientId, Listener listener);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_double_value")]
        public static extern byte Set(ushort clientId, ref CheetahObjectId objectId, ushort fieldId, double value);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "inc_double_value")]
        public static extern byte Increment(ushort clientId, ref CheetahObjectId objectId, ushort fieldId, double increment);
    }
}