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
  display: none;
  position: absolute;
  bottom: -2px;
  left: 50%;
  width: 100%;
  transform: translateX(-50%);
  background: #258e39;
  color: #ffffffdb;
  padding: 4px;
  font-weight: 500;
  font-size: 13px;
  line-height: 13px;
  white-space: nowrap;
`;

const StyledIcon = styled.button<{ selected: boolean }>`
  position: relative;
  overflow: hidden;
  border-radius: 12px;
  padding: 17px;
  border: 2px solid transparent;
  background: ${(props) => (props.selected ? `#4dff6e1a` : `transparent`)};
  border-color: ${(props) => (props.selected ? `#37c852` : 'transparent')};
  cursor: ${(props) => (props.selected ? `auto` : `pointer`)};
  height: 100px;
  width: 100px;
  color: ${(props) => (props.selected ? '#2cff54e6' : '#ffffffa6')};
  transition: all 0.2s ease;

  &:hover {
    background: #4dff6e1a;
    border-color: #37c852;
    padding: ${(props) => (props.selected ? '17px' : '5px 17px 40px')};
    color: ${(props) => (props.selected ? '#2cff54e6' : '#c8ffd3e6')};

    ${SetAsIcon} {
      display: ${(props) => (props.selected ? 'none' : 'block')};
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

const Resolution = styled.span`
  display: flex;
  align-items: center;
  white-space: pre;
  position: absolute;
  bottom: 2px;
  left: 50%;
  transform: translateX(-50%);
  text-align: center;
  font-weight: 600;
  font-family: system-ui;
  padding: 2px 5px;
  line-height: 9px;
  font-size: 10px;

  &:after {
    content: '';
    background: currentColor;
    opacity: 0.18;
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    border-radius: 3px;
  }
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

      <Resolution>
        {type === 'svg' ? 'SVG' : type === 'ico' ? sizes[0] : size}
      </Resolution>

      <SetAsIcon>
        Set as icon
        <br />
        in PR
      </SetAsIcon>
    </StyledIcon>
  );
}
