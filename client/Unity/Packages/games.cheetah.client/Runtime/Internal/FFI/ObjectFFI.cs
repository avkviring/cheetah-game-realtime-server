using System.Runtime.InteropServices;
using Games.Cheetah.Client.ServerAPI;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.Internal.FFI
{
    internal static class ObjectFFI
    {
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_create_object_listener")]
        public static extern byte SetCreateListener(ushort clientId, IObjectServerAPI.CreateListener listener);


        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_created_object_listener")]
        public static extern byte SetCreatedListener(ushort clientId, IObjectServerAPI.CreatedListener listener);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "create_object")]
        public static extern byte CreateObject(ushort clientId, ushort template, ulong accessGroup, ref CheetahObjectId objectId);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "created_object")]
        public static extern byte Created(ushort clientId, in CheetahObjectId objectId, bool roomOwner, ref CheetahBuffer singletonKey);


        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_delete_object_listener")]
        public static extern byte SetDeleteListener(ushort clientId, IObjectServerAPI.DeleteListener objectDeleteListener);


        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "delete_object")]
        public static extern byte Delete(ushort clientId, in CheetahObjectId objectId);
    }
}