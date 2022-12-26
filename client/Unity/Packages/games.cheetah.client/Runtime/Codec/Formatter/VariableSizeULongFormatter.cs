using System;
using System.Runtime.CompilerServices;
using Games.Cheetah.Client.Types;
using Games.Cheetah.Client.Types.Field;

namespace Games.Cheetah.Client.Codec.Formatter
{
    public sealed class VariableSizeULongFormatter : Formatter<ulong>, FixedArrayFormatter<ulong>, ArrayFormatter<ulong>
    {
        public static readonly VariableSizeULongFormatter Instance = new();


        private const byte U8MAX = 249;
        private const byte U9Marker = 250;
        private const byte U16Marker = 251;
        private const byte U24Marker = 252;
        private const byte U32Marker = 253;
        private const byte U48Marker = 254;
        private const byte U64Marker = 255;


        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public ulong Read(ref NetworkBuffer buffer)
        {
            return StaticRead(ref buffer);
        }


        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public void Write(ulong value, ref NetworkBuffer buffer)
        {
            StaticWrite(value, ref buffer);
        }


        public unsafe void ReadFixedArray(ref NetworkBuffer buffer, ulong* value, uint size, uint offset)
        {
            for (var i = 0; i < size; i++)
            {
                value[i + offset] = Read(ref buffer);
            }
        }

        public unsafe void WriteFixedArray(ulong* value, uint size, uint offset, ref NetworkBuffer buffer)
        {
            for (var i = 0; i < size; i++)
            {
                Write(value[i + offset], ref buffer);
            }
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public void ReadArray(ref NetworkBuffer buffer, ulong[] value, uint size, uint offset)
        {
            for (var i = 0; i < size; i++)
            {
                value[i + offset] = Read(ref buffer);
            }
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public void WriteArray(ulong[] value, uint size, uint offset, ref NetworkBuffer buffer)
        {
            for (var i = 0; i < size; i++)
            {
                Write(value[i + offset], ref buffer);
            }
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static unsafe ulong StaticRead(ref NetworkBuffer buffer)
        {
            buffer.AssertEnoughData(1);
            var first = buffer.values[buffer.pos++];
            switch (first)
            {
                case < U8MAX:
                    return first;
                case U9Marker:
                    buffer.AssertEnoughData(1);
                    return (ulong)(buffer.values[buffer.pos++] + U8MAX);
                case U16Marker:
                    buffer.AssertEnoughData(2);
                    return UShortFormatter.StaticUncheckedRead(ref buffer);
                case U24Marker:
                    buffer.AssertEnoughData(3);
                    return (ulong)((buffer.values[buffer.pos++] << 16) + UShortFormatter.StaticUncheckedRead(ref buffer));
                case U32Marker:
                    buffer.AssertEnoughData(4);
                    return UIntFormatter.StaticUncheckedRead(ref buffer);
                case U48Marker:
                    buffer.AssertEnoughData(6);
                    return ((ulong)buffer.values[buffer.pos++] << 40)
                           + ((ulong)buffer.values[buffer.pos++] << 32)
                           + UIntFormatter.StaticUncheckedRead(ref buffer);
                case U64Marker:
                    buffer.AssertEnoughData(8);
                    return ULongFormatter.StaticUncheckedRead(ref buffer);
            }

            throw new Exception("Unknown marker " + first + ".");
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static unsafe void StaticWrite(ulong value, ref NetworkBuffer buffer)
        {
            switch (value)
            {
                case < U8MAX:
                    buffer.AssertFreeSpace(1);
                    buffer.values[buffer.size++] = (byte)(value & 0xFF);
                    break;
                case < U8MAX + 255:
                    buffer.AssertFreeSpace(2);
                    buffer.values[buffer.size++] = U9Marker;
                    buffer.values[buffer.size++] = (byte)(value - U8MAX);
                    break;
                case <= ushort.MaxValue:
                    buffer.AssertFreeSpace(3);
                    buffer.values[buffer.size++] = U16Marker;
                    UShortFormatter.StaticUncheckedWrite((ushort)value, ref buffer);
                    break;
                case <= ushort.MaxValue * byte.MaxValue:
                    buffer.AssertFreeSpace(4);
                    buffer.values[buffer.size++] = U24Marker;
                    buffer.values[buffer.size++] = (byte)((value & 0xFF0000) >> 16);
                    UShortFormatter.StaticUncheckedWrite((ushort)(value & 0xFFFF), ref buffer);
                    break;
                case <= uint.MaxValue:
                    buffer.AssertFreeSpace(5);
                    buffer.values[buffer.size++] = U32Marker;
                    UIntFormatter.StaticUncheckedWrite((uint)value, ref buffer);
                    break;
                case <= (ulong)uint.MaxValue * byte.MaxValue * byte.MaxValue:
                    buffer.AssertFreeSpace(7);
                    buffer.values[buffer.size++] = U48Marker;
                    buffer.values[buffer.size++] = (byte)((value & 0xFF0000000000) >> 40);
                    buffer.values[buffer.size++] = (byte)((value & 0xFF00000000) >> 32);
                    UIntFormatter.StaticUncheckedWrite((uint)(value & 0xFFFFFFFF), ref buffer);
                    break;
                default:
                    buffer.AssertFreeSpace(9);
                    buffer.values[buffer.size++] = U64Marker;
                    ULongFormatter.StaticUncheckedWrite(value, ref buffer);
                    break;
            }
        }
    }
}