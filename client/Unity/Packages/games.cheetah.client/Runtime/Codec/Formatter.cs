using System.Runtime.CompilerServices;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.Codec
{
    /// <summary>
    /// Преобразование данных в бинарный поток и обратно, отличается от Codec тем, что не требует reference и предназначен непосредственно для работы с буфером
    /// </summary>
    /// <typeparam name="T"></typeparam>
    public interface Formatter<T> : Codec<T>
    {
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public T Read(ref CheetahBuffer buffer);

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public void Write(T value, ref CheetahBuffer buffer);

        void Codec<T>.Decode(ref CheetahBuffer buffer, ref T dest)
        {
            dest = Read(ref buffer);
        }

        void Codec<T>.Encode(in T source, ref CheetahBuffer buffer)
        {
            Write(source, ref buffer);
        }
    }


    public interface FixedArrayFormatter<T> where T : unmanaged
    {
        public unsafe void ReadFixedArray(ref CheetahBuffer buffer, T* value, uint size, uint offset);
        public unsafe void WriteFixedArray(T* value, uint size, uint offset, ref CheetahBuffer buffer);
    }

    public interface ArrayFormatter<in T>
    {
        public void ReadArray(ref CheetahBuffer buffer, T[] value, uint size, uint offset);
        public void WriteArray(T[] value, uint size, uint offset, ref CheetahBuffer buffer);
    }

    public abstract class UnmanagedFormatter<T> : Formatter<T>, ArrayFormatter<T>, FixedArrayFormatter<T> where T : unmanaged
    {
        public unsafe T Read(ref CheetahBuffer buffer)
        {
            buffer.AssertEnoughData((uint)sizeof(T));
            return UncheckedRead(ref buffer);
        }


        public unsafe void Write(T value, ref CheetahBuffer buffer)
        {
            buffer.AssertFreeSpace((uint)sizeof(T));
            UncheckedWrite(value, ref buffer);
        }

        public unsafe void ReadFixedArray(ref CheetahBuffer buffer, T* value, uint size, uint offset)
        {
            buffer.AssertEnoughData((uint)(size * sizeof(T)));
            for (var i = 0; i < size; i++)
            {
                value[i + offset] = Read(ref buffer);
            }
        }

        public unsafe void WriteFixedArray(T* value, uint size, uint offset, ref CheetahBuffer buffer)
        {
            buffer.AssertFreeSpace((uint)(size * sizeof(T)));
            for (var i = 0; i < size; i++)
            {
                Write(value[i + offset], ref buffer);
            }
        }

        /// <summary>
        /// Чтение без проверки доступных данных в буфере
        /// </summary>
        /// <param name="buffer"></param>
        /// <returns></returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public abstract T UncheckedRead(ref CheetahBuffer buffer);

        /// <summary>
        /// Запись без проверки свободного места в буфере
        /// </summary>
        /// <param name="value"></param>
        /// <param name="buffer"></param>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public abstract void UncheckedWrite(T value, ref CheetahBuffer buffer);

        public unsafe void ReadArray(ref CheetahBuffer buffer, T[] value, uint size, uint offset)
        {
            buffer.AssertEnoughData((uint)(size * sizeof(T)));
            for (var i = 0; i < size; i++)
            {
                value[i + offset] = Read(ref buffer);
            }
        }

        public unsafe void WriteArray(T[] value, uint size, uint offset, ref CheetahBuffer buffer)
        {
            buffer.AssertFreeSpace((uint)(size * sizeof(T)));
            for (var i = 0; i < size; i++)
            {
                Write(value[i + offset], ref buffer);
            }
        }
    }
}