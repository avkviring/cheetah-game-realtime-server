using System.Runtime.InteropServices;
using Games.Cheetah.Client.ServerAPI;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.Internal.FFI
{
    internal static class FieldFFI
    {
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_delete_field_listener")]
        public static extern byte SetListener(ushort clientId, IFieldServerAPI.Listener listener);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "delete_field")]
        public static extern byte Delete(ushort clientId, in CheetahObjectId objectId, ushort fieldId, FieldType value);
    }
}