using System.Collections.Generic;
using System.Runtime.CompilerServices;
using Cheetah.Matches.Realtime.Tests.Generator;
using Games.Cheetah.Client.Codec;
using Games.Cheetah.Client.Editor.Generator;
using Games.Cheetah.Client.Editor.Generator.Fields.Array;
using Games.Cheetah.Client.Editor.Generator.Fields.Array.Exceptions;
using NUnit.Framework;

namespace Games.Cheetah.Client.Tests.Generator.Fields.Array
{
    public class TestFixedArrayFieldCodecGenerator
    {
        [Test]
        public void ShouldNullIfNotArray()
        {
            Assert.Null(FixedArrayFieldGenerator.Create(
                new Formatters(),
                new FieldInfoStubBuilder<int>("name").Build(),
                new HashSet<string>(),
                new HashSet<string>()));
        }


        [Test]
        public void ShouldFailWhenMissingArraySizeField()
        {
            Assert.Throws<MissingArraySizeFieldException>(() =>
            {
                FixedArrayFieldGenerator.Create(
                    new Formatters(),
                    CreateFieldInfoAccessor(),
                    new HashSet<string>(),
                    new HashSet<string>());
            });
        }

        [Test]
        public void ShouldFailWhenIncorrectOrderArraySizeField()
        {
            Assert.Throws<IncorrectOrderArraySizeFieldException>(() =>
            {
                FixedArrayFieldGenerator.Create(
                    new Formatters(),
                    CreateFieldInfoAccessor(),
                    new HashSet<string>(),
                    new HashSet<string> { "size" });
            });
        }

        [Test]
        public void ShouldThrowUnsupportedType()
        {
            Assert.Throws<FixedArrayUnsupportedTypeException>(() =>
            {
                var builder = new FieldInfoStubBuilder<int[]>("name");
                builder.AddAttribute(new ArraySizeField("size"));
                builder.AddAttribute(new FixedBufferAttribute(typeof(string), 10));
                var fieldInfoAccessorStub = builder.Build();
                FixedArrayFieldGenerator.Create(
                    new Formatters(),
                    fieldInfoAccessorStub,
                    new HashSet<string> { "size" },
                    new HashSet<string> { "size" });
            });
        }

        [Test]
        public void ShouldCreate()
        {
            var generator = FixedArrayFieldGenerator.Create(
                new Formatters(),
                CreateFieldInfoAccessor(),
                new HashSet<string> { "size" },
                new HashSet<string> { "size" });
            Assert.NotNull(generator);
        }

        private static FieldInfoAccessorStub CreateFieldInfoAccessor()
        {
            var builder = new FieldInfoStubBuilder<int[]>("name");
            builder.AddAttribute(new ArraySizeField("size"));
            builder.AddAttribute(new FixedBufferAttribute(typeof(int), 10));
            var fieldInfoAccessorStub = builder.Build();
            return fieldInfoAccessorStub;
        }
    }
}