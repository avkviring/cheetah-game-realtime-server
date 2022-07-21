using System.Runtime.InteropServices;
using Cheetah.Matches.Relay.Types;

namespace Cheetah.Matches.Relay.Internal.FFI
{
    internal class StructureFFI
    {
        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void Listener(ushort commandCreator, ref CheetahObjectId objectId, ushort fieldId, ref CheetahBuffer data);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_structure_listener")]
        public static extern byte SetListener(ushort clientId, Listener listener);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_structure")]
        public static extern byte Set(ushort clientId, ref CheetahObjectId objectId, ushort fieldId, ref CheetahBuffer data);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "compare_and_set_structure")]
        public static extern byte CompareAndSet(
            ushort clientId,
            ref CheetahObjectId objectId,
            ushort fieldId,
            ref CheetahBuffer currentValue,
            ref CheetahBuffer newValue,
            bool hasReset,
            ref CheetahBuffer resetValue
        );
    }
}