using System;
using Games.Cheetah.Client.Codec;

namespace Games.Cheetah.Client.Editor.Generator.Fields.Array.Exceptions
{
    internal class VariableSizeTypeNotSupportedException : Exception
    {
        public VariableSizeTypeNotSupportedException(Type type) : base("Type " + type.Name + " not supported with " + nameof(VariableSizeCodec))
        {
        }
    }
}