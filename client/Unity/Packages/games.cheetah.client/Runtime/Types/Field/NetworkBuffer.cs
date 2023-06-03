using System;
using System.Runtime.InteropServices;
using System.Text;
using Games.Cheetah.Client.Codec;
using Games.Cheetah.Client.Internal.FFI;
using UnityEngine;

namespace Games.Cheetah.Client.Types.Field
{
    [StructLayout(LayoutKind.Sequential)]
    public unsafe struct NetworkBuffer
    {
        public ushort size;
        public ushort pos;

        [MarshalAs(UnmanagedType.LPArray)] public fixed byte values[FFIMethods.MaxSizeStruct];


        public NetworkBuffer(byte[] source) : this()
        {
            foreach (var b in source)
            {
                Add(b);
            }
        }

        private NetworkBuffer Add(byte value)
        {
            values[size] = value;
            size++;
            return this;
        }

        public NetworkBuffer Add(byte[] value)
        {
            foreach (var b in value)
            {
                values[size] = b;
                size++;
            }

            return this;
        }


        public override string ToString()
        {
            var builder = new StringBuilder();
            builder.Append($"Bytes[size = {size},pos = {pos}, data=(");
            for (var i = 0; i < size; i++)
            {
                if (i > 0)
                {
                    builder.Append(" ");
                }

                builder.Append(values[i].ToString("X2"));
            }

            builder.Append(")]");

            return builder.ToString();
        }

        public void Clear()
        {
            size = 0;
        }

        public void AssertEnoughData(uint readSize)
        {
            if (pos + readSize > size)
            {
                Debug.LogError(pos + " " + readSize + " " + size);
                throw new EndOfBufferException();
            }
        }

        public void AssertFreeSpace(uint space)
        {
            if (size + space > FFIMethods.MaxSizeStruct)
            {
                throw new IndexOutOfRangeException();
            }
        }


        internal class EndOfBufferException : Exception
        {
        }
    }

    public static class NetworkBufferExtensions
    {
        public static NetworkBuffer ToNetworkBuffer<T>(this T item, CodecRegistry codecRegistry) where T : struct
        {
            var result = new NetworkBuffer();
            codecRegistry.GetCodec<T>().Encode(in item, ref result);
            return result;
        }
    }
}