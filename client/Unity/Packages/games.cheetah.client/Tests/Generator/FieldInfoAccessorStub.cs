using System;
using System.Collections.Generic;
using Games.Cheetah.Client.Editor.Generator;

namespace Cheetah.Matches.Realtime.Tests.Generator
{
    public class FieldInfoStubBuilder<T>
    {
        private readonly string name;
        private readonly List<object> attributes = new();

        public FieldInfoStubBuilder(string name)
        {
            this.name = name;
        }

        public FieldInfoAccessorStub Build()
        {
            return new FieldInfoAccessorStub(name, typeof(T), attributes);
        }

        public void AddAttribute(Attribute attribute)
        {
            attributes.Add(attribute);
        }
    }


    public class FieldInfoAccessorStub : FieldInfoAccessor
    {
        private readonly List<object> attributes;

        public FieldInfoAccessorStub(string name, Type type, List<object> attributes)
        {
            Name = name;
            FieldType = type;
            this.attributes = attributes;
        }

        public T GetCustomAttribute<T>() where T : Attribute
        {
            return (T)attributes.Find(a => a.GetType() == typeof(T));
        }

        public Type FieldType { get; }
        public string Name { get; }
    }
}