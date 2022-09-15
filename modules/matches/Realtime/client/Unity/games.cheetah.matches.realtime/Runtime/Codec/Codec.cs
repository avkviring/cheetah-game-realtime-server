using Cheetah.Matches.Realtime.Types;

namespace Cheetah.Matches.Realtime.Codec
{
    /// <summary>
    /// Интерфейс сериализации объектов для взаимодействия между клиентом и сервером
    /// </summary>
    /// <typeparam name="T"></typeparam>
    public interface Codec<T>
    {
        void Decode(ref CheetahBuffer buffer, ref T dest);
        void Encode(in T source, ref CheetahBuffer buffer);
    }
}