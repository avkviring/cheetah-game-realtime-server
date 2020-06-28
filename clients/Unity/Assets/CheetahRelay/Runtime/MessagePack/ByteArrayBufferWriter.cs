using System;
using System.Buffers;

namespace CheetahRelay.Tests.Runtime
{
    public class ByteArrayBufferWriter : IBufferWriter<byte>
    {
        private readonly byte[] _array;
        public int Count;

        public ByteArrayBufferWriter(byte[] array)
        {
            this._array = array;
        }

        public void Advance(int count)
        {
            this.Count += count;
        }

        public Memory<byte> GetMemory(int sizeHint = 0)
        {
            var memory = new Memory<byte>(_array);
            return memory;
        }

        public Span<byte> GetSpan(int sizeHint = 0)
        {
            return new Span<byte>(_array);
        }
    }
}