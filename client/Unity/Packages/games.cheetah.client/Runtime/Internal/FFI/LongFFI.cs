using System.Runtime.InteropServices;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.Internal.FFI
{
    internal static class LongFFI
    {
        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void Listener(ushort commandCreator, in CheetahObjectId objectId, ushort fieldId, long value);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_long_value_listener")]
        public static extern byte SetListener(ushort clientId, Listener listener);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_long_value")]
        public static extern byte Set(ushort clientId, in CheetahObjectId objectId, ushort fieldId, long value);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "inc_long_value")]
        public static extern byte Increment(ushort clientId, in CheetahObjectId objectId, ushort fieldId, long increment);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "compare_and_set_long_value")]
        public static extern byte CompareAndSet(ushort clientId, in CheetahObjectId objectId, ushort fieldId, long currentValue, long newValue,
            bool hasReset, long resetValue);
    }
}