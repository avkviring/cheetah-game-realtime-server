using System.Runtime.InteropServices;
using Games.Cheetah.Client.ServerAPI;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.Internal.FFI
{
    public class StructureFFI
    {
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_structure_listener")]
        public static extern byte SetListener(ushort clientId, IStructureServerAPI.Listener listener);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_structure")]
        public static extern byte Set(ushort clientId, in CheetahObjectId objectId, ushort fieldId, ref CheetahBuffer data);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "compare_and_set_structure")]
        public static extern byte CompareAndSet(
            ushort clientId,
            in CheetahObjectId objectId,
            ushort fieldId,
            ref CheetahBuffer currentValue,
            ref CheetahBuffer newValue,
            bool hasReset,
            ref CheetahBuffer resetValue
        );
    }
}