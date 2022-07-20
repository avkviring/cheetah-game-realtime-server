using Cheetah.Matches.Relay.Codec;
using Cheetah.Matches.Relay.Types;
using NUnit.Framework;

namespace Cheetah.Matches.Relay.Tests.Runtime.Codec.Types
{
    public abstract class AbstractFieldTest<T> where T : new()
    {
        [Test]
        public void Test()
        {
            var codecRegistryBuilder = new CodecRegistryBuilder();
            var codecRegistry = codecRegistryBuilder.Build();
            var codec = codecRegistry.GetCodec<T>();
            var source = GetSource();
            var buffer = new CheetahBuffer();
            codec.Encode(ref source, ref buffer);
            var result = GetResult();
            buffer.pos = 0;
            codec.Decode(ref buffer, ref result);
            CheckResult(source, result);
        }

        protected abstract void CheckResult(T source, T result);


        protected virtual T GetResult()
        {
            return new T();
        }


        protected abstract T GetSource();
    }
}