using CheetahRelay.Runtime.Client.Codec;
using NUnit.Framework;

namespace CheetahRelay.Tests
{
    public class MessagePackTest
    {
        private const ushort FieldId = 10;

        [Test]
        public  void ShouldSerializeStruct()
        {
            var codecRegistry = new CodecRegistry();
            codecRegistry.RegisterStructure(FieldId, new MessagePackCodec<SomeStructure>());
            var originalMessage = new SomeStructure {Age = 100500};


            var buffer = new RelayBuffer();
            codecRegistry.EncodeStructure(FieldId, originalMessage, ref buffer);
            var message = (SomeStructure) codecRegistry.DecodeStructure(FieldId, ref buffer);
            
            Assert.AreEqual(originalMessage, message);
        }
    }
}