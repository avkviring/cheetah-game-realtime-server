using System.Runtime.InteropServices;
using UnityEngine;
using UnityEngine.UIElements;

namespace Cheetach.Relay
{
    public class NativeClient
    {
#if UNITY_IOS || UNITY_TVOS
        private const string Import = "__Internal";
#else
        private const string Import = "librelay";
#endif

        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void CallbackDelegate(TestStruct x);

        public event CallbackDelegate Event;

        [DllImport(dllName: Import, CallingConvention = CallingConvention.Cdecl)]
        private static extern void init(CallbackDelegate c);

        
        [DllImport(dllName: Import, CallingConvention = CallingConvention.Cdecl)]
        private static extern void CollectFromNative();


        public NativeClient()
        {
            init(Event);
        }
        
        public void Collect()
        {
            CollectFromNative();
        }
    }

    public struct TestStruct
    {
        public double id;
    }
}