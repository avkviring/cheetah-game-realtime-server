using System;
using System.Runtime.InteropServices;

namespace Cheetach.Relay
{
    public class NativeClient
    {
#if UNITY_IOS || UNITY_TVOS
        private const string Import = "__Internal";
#else
        private const string Import = "libcheetah_relay_client";
#endif

        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void CallbackDelegate(ref S2CCommand command);

        public event CallbackDelegate Event;

        [DllImport(dllName: Import, CallingConvention = CallingConvention.Cdecl)]
        private static extern void init(S2CCommand command);


        [DllImport(dllName: Import, CallingConvention = CallingConvention.Cdecl)]
        private static extern void CollectFromNative();

        [DllImport(dllName: Import, CallingConvention = CallingConvention.Cdecl)]
        public static extern void test(CallbackDelegate e);


        public NativeClient()
        {
        }

        public void Collect()
        {
            //CollectFromNative();
        }
    }

    public struct TestStruct
    {
        public double id;
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct S2CCommand
    {
        public S2CCommandType commandType;
        public UploadObjectS2C upload;
    }


    public enum S2CCommandType : byte
    {
        Upload,
        SetLongCounter,
        SetFloatCounter,
        SetStruct,
        ReceiveEvent,
        Unload,
    }


    [StructLayout(LayoutKind.Sequential)]
    public struct UploadObjectS2C
    {
        public UInt64 id;
        
        public UInt16 long_counters_count;
        [MarshalAs(UnmanagedType.ByValArray, SizeConst = 1024)]
        public Field<Int64>[] long_counters;


        // pub long_counters_count: u16,
        // pub long_counters_fields: *const u16,
        // pub long_counters_values: *const i64,
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct Field<T>
    {
        public UInt16 field_id;
        public T value;
    }
}