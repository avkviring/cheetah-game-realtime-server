using System;
using Cheetah.Matches.Relay.Codec;

namespace Cheetah.Matches.Relay.Editor.Generator.Fields.Array.Exceptions
{
    internal class VariableSizeTypeNotSupportedException : Exception
    {
        public VariableSizeTypeNotSupportedException(Type type) : base("Type " + type.Name + " not supported with " + nameof(VariableSizeCodec))
        {
        }
    }
}