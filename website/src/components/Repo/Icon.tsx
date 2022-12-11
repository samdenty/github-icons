import styled from '@emotion/styled';
import { useQuery } from 'react-query';

export type IconInfo =
  | {
      type: 'png' | 'jpeg';
      size: string;
      sizes?: undefined;
    }
  | {
      type: 'ico';
      size?: undefined;
      sizes: string[];
    }
  | { type: 'svg'; size?: undefined; sizes?: undefined };

export type Icon = IconInfo & {
  url: string;
  headers: Record<string, string>;

  kind:
    | 'icon_field'
    | 'user_avatar'
    | 'app_icon'
    | 'blob'
    | 'readme_image'
    | 'site_logo'
    | 'site_favicon';
};

const SetAsIcon = styled.div`
  opacity: 0;
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  background: rgb(55 200 82 / 66%);
  backdrop-filter: blur(15px);
  color: white;
  border: 1px solid rgba(240, 246, 252, 0.1);
  padding: 3px 8px;
  border-radius: 6px;
  font-weight: 500;
  font-size: 13px;
  line-height: 20px;
  white-space: nowrap;
`;

const StyledIcon = styled.button<{ selected: boolean }>`
  position: relative;
  border-radius: 16px;
  padding: 17px;
  border: 3px solid transparent;
  border-color: ${(props) => (props.selected ? `#007aff` : 'transparent')};
  background: ${(props) => (props.selected ? `#007aff21` : `transparent`)};
  cursor: ${(props) => (props.selected ? `auto` : `pointer`)};
  height: 150px;
  width: 150px;
  color: ${(props) => (props.selected ? '#308ff7' : '#37c852')};

  &:hover {
    background: ${(props) => (props.selected ? `#007aff21` : `#ffffff21`)};
    border-color: ${(props) => (props.selected ? `#007aff` : 'rgb(55 200 82)')};

    ${SetAsIcon} {
      opacity: ${(props) => (props.selected ? 0 : 1)};
    }

    img {
      opacity: ${(props) => (props.selected ? 1 : 0.3)};
    }
  }

  img {
    display: block;
    width: 100%;
    height: 100%;
    aspect-ratio: 1;
    object-fit: contain;
  }

  &:empty {
    display: none;
  }
`;

const Info = styled.div`
  display: flex;
  align-items: center;
  white-space: pre;
  position: absolute;
  bottom: 2px;
  left: 50%;
  transform: translateX(-50%);
  text-align: center;
  font-family: system-ui;
  padding: 2px 5px;
  background: #ffffff12;
  border-radius: 3px;
  line-height: 9px;
  font-size: 10px;
`;

const Kind = styled.span`
  color: #ffffff6b;
  font-weight: 400;
`;

const Resolution = styled.span`
  font-weight: 600;
`;

type IconProps = Icon & {
  selected?: boolean;
};

export function Icon({
  url,
  headers,
  type,
  kind,
  size,
  sizes,
  selected = false,
}: IconProps) {
  const hasHeaders = Object.keys(headers).length !== 0;

  if (hasHeaders) {
    const { data } = useQuery(
      [url, headers],
      () =>
        fetch(url, { headers })
          .then((res) => res.blob())
          .then(URL.createObjectURL),
      { cacheTime: 0 }
    );

    url = data!;
  }

  return (
    <StyledIcon selected={selected} onClick={() => {}}>
      <img src={url} />

      <SetAsIcon>Set as Icon in PR</SetAsIcon>

      <Info>
        <Kind>{kind} — </Kind>
        <Resolution>
          {type === 'svg' ? 'SVG' : type === 'ico' ? sizes[0] : size}
        </Resolution>
      </Info>
    </StyledIcon>
  );
}
