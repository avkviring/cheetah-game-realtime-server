using System;
using MessagePack;
using MessagePack.Resolvers;

namespace CheetahRelay.Runtime.Client.Codec
{
    public class MessagePackCodecFactory
    {
        public MessagePackCodecFactory(IFormatterResolver resolver)
        {
            StaticCompositeResolver.Instance.Register(resolver, StandardResolver.Instance);
            var option = MessagePackSerializerOptions.Standard.WithResolver(StaticCompositeResolver.Instance);
            MessagePackSerializer.DefaultOptions = option;
        }

        public Codec Create<T>()
        {
            return new MessagePackCodec<T>();
        }
    }

    public class MessagePackCodec<T> : Codec
    {
        private static byte[] buffer = new byte[1024];
        private static ByteArrayBufferWriter bufferForWrite = new ByteArrayBufferWriter(buffer);


        public MessagePackCodec()
        {
            MessagePackSerializer.DefaultOptions.Resolver.GetFormatter<T>();
        }

        public object Decode(ref RelayBuffer relayBuffer)
        {
            unsafe
            {
                fixed (byte* values = relayBuffer.values)
                {
                    var span = new Span<byte>(values, relayBuffer.size);
                    //TODO - избавиться от копирования
                    return MessagePackSerializer.Deserialize<T>(new ReadOnlyMemory<byte>(span.ToArray()), MessagePackSerializer.DefaultOptions);
                }
            }
        }

        public void Encode(object value, ref RelayBuffer buffer)
        {
            bufferForWrite.Clear();

            MessagePackSerializer.Serialize(bufferForWrite, (T) value);

            var span = bufferForWrite.GetSpan();
            var count = bufferForWrite.Count;
            //TODO избавиться от копирования
            for (var i = 0; i < count; i++)
            {
                buffer.Add(span[i]);
            }
        }
    }
}